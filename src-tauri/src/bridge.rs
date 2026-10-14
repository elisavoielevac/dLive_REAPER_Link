//! One backend owner for the selected scene source. No window timer owns a connection.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    net::{IpAddr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tauri::Emitter;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub transport: String,
    pub host: String,
    pub port: u16,
    pub channel: u8,
    pub midi_input: String,
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=12).contains(&self.channel) {
            return Err("Desk MIDI channel must be between 1 and 12".into());
        }
        match self.transport.as_str() {
            "midi" => Ok(()),
            "tcp" => {
                self.host
                    .parse::<IpAddr>()
                    .map_err(|_| "Enter a MixRack IP address")?;
                if self.port == 0 {
                    return Err("Port must be between 1 and 65535".into());
                }
                if matches!(self.port, 51327..=51329) {
                    return Err(
                        "Use the plain MixRack port 51325. Surface and TLS ports are unsupported"
                            .into(),
                    );
                }
                Ok(())
            }
            _ => Err("Unknown scene source".into()),
        }
    }
    fn configured(&self) -> bool {
        if self.transport == "tcp" {
            !self.host.is_empty()
        } else {
            !self.midi_input.is_empty()
        }
    }
}
#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub config: Config,
    pub requested: bool,
    pub status: String,
    pub detail: String,
    pub revision: u64,
}
struct Control {
    snapshot: Snapshot,
    exit: bool,
    session: Option<u64>,
    tcp_writer: Option<TcpStream>,
}
// A pending recall belongs to this connection, never a later reconnect.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct SendSession {
    revision: u64,
    session: u64,
}

// A&H V2: BN 00 bank, CN program. Explicit status and bank on every recall.
fn scene_recall_bytes(scene: u16, channel: u8) -> Result<[u8; 5], String> {
    if !(1..=500).contains(&scene) {
        return Err("Scene number must be between 1 and 500".into());
    }
    if !(1..=12).contains(&channel) {
        return Err("Desk MIDI channel must be between 1 and 12".into());
    }
    let index = scene - 1;
    Ok([
        0xb0 + channel - 1,
        0,
        (index / 128) as u8,
        0xc0 + channel - 1,
        (index % 128) as u8,
    ])
}

pub struct Bridge {
    control: Arc<Mutex<Control>>,
    path: PathBuf,
}
impl Bridge {
    pub fn load(path: PathBuf, midi_input: String) -> Result<Config, String> {
        match fs::read_to_string(path) {
            Ok(text) => {
                let config: Config = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                config.validate()?;
                Ok(config)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config {
                transport: "midi".into(),
                host: String::new(),
                port: 51325,
                channel: 12,
                midi_input,
            }),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn new(
        config: Config,
        path: PathBuf,
        app: Option<tauri::AppHandle>,
        mut handle_message: impl FnMut(SceneRecall) + Send + 'static,
    ) -> Self {
        let requested = config.configured();
        let control = Arc::new(Mutex::new(Control {
            snapshot: Snapshot {
                config,
                requested,
                status: if requested { "waiting" } else { "stopped" }.into(),
                detail: String::new(),
                revision: 0,
            },
            exit: false,
            session: None,
            tcp_writer: None,
        }));
        let (tx, rx) = mpsc::sync_channel::<(u64, u64, SceneRecall)>(128);
        let dispatch_control = control.clone();
        thread::spawn(move || {
            while let Ok((revision, session, recall)) = rx.recv() {
                let active = {
                    let c = dispatch_control.lock().unwrap();
                    !c.exit
                        && c.snapshot.requested
                        && c.snapshot.revision == revision
                        && c.session == Some(session)
                };
                if active {
                    handle_message(recall);
                }
            }
        });
        let worker_control = control.clone();
        thread::spawn(move || worker(worker_control, tx, app));
        Self { control, path }
    }
    pub fn snapshot(&self) -> Snapshot {
        self.control.lock().unwrap().snapshot.clone()
    }
    // Internal only; the playback worker owns arming and scene decisions.
    pub(crate) fn send_session(&self) -> Result<SendSession, String> {
        let c = self
            .control
            .lock()
            .map_err(|_| "Scene source unavailable")?;
        if c.exit
            || !c.snapshot.requested
            || c.snapshot.config.transport != "tcp"
            || c.snapshot.status != "connected"
            || c.tcp_writer.is_none()
        {
            return Err("Scene sending requires an active Direct MixRack TCP connection".into());
        }
        Ok(SendSession {
            revision: c.snapshot.revision,
            session: c.session.ok_or("MixRack connection unavailable")?,
        })
    }

    pub(crate) fn send_scene(&self, expected: SendSession, scene: u16) -> Result<(), String> {
        let mut c = self
            .control
            .lock()
            .map_err(|_| "Scene source unavailable")?;
        if c.exit
            || !c.snapshot.requested
            || c.snapshot.config.transport != "tcp"
            || c.snapshot.status != "connected"
            || c.snapshot.revision != expected.revision
            || c.session != Some(expected.session)
        {
            return Err("MixRack connection changed; scene recall discarded".into());
        }
        let bytes = scene_recall_bytes(scene, c.snapshot.config.channel)?;
        // Serialize short, bounded writes with control changes. No queue/retry.
        let result = c
            .tcp_writer
            .as_mut()
            .ok_or("MixRack connection unavailable")?
            .write_all(&bytes);
        if let Err(error) = result {
            if let Some(stream) = c.tcp_writer.take() {
                let _ = stream.shutdown(Shutdown::Both);
            }
            c.session = None;
            c.snapshot.revision += 1;
            c.snapshot.status = "waiting".into();
            c.snapshot.detail = format!("Scene send failed; delivery unknown: {error}");
            return Err(c.snapshot.detail.clone());
        }
        // This confirms only the socket write, not that the desk recalled a scene.
        Ok(())
    }

    pub fn save(&self, mut config: Config) -> Result<(), String> {
        config.host = config.host.trim().into();
        config.validate()?;
        let mut c = self.control.lock().unwrap();
        // Save first; a failed save must not change the live connection.
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(
            &self.path,
            serde_json::to_string(&config).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if c.snapshot.config != config {
            c.tcp_writer = None;
            c.session = None;
            c.snapshot.config = config;
            c.snapshot.revision += 1;
            c.snapshot.status = if c.snapshot.requested {
                "waiting"
            } else {
                "stopped"
            }
            .into();
            c.snapshot.detail.clear();
        }
        Ok(())
    }
    pub fn listen(&self, requested: bool) {
        let mut c = self.control.lock().unwrap();
        c.tcp_writer = None;
        c.session = None;
        c.snapshot.requested = requested;
        c.snapshot.revision += 1;
        c.snapshot.status = if requested { "waiting" } else { "stopped" }.into();
        c.snapshot.detail.clear();
    }
    pub fn reconnect(&self) {
        let mut c = self.control.lock().unwrap();
        if c.snapshot.requested {
            c.tcp_writer = None;
            c.session = None;
            c.snapshot.revision += 1;
            c.snapshot.status = "waiting".into();
            c.snapshot.detail.clear();
        }
    }
}
impl Drop for Bridge {
    fn drop(&mut self) {
        self.control.lock().unwrap().exit = true;
    }
}
fn report(control: &Mutex<Control>, revision: u64, status: &str, detail: String) {
    let mut c = control.lock().unwrap();
    if c.snapshot.revision == revision && c.snapshot.requested {
        c.snapshot.status = status.into();
        c.snapshot.detail = detail;
    }
}
fn worker(
    control: Arc<Mutex<Control>>,
    tx: mpsc::SyncSender<(u64, u64, SceneRecall)>,
    app: Option<tauri::AppHandle>,
) {
    let mut revision = u64::MAX;
    let mut session = 0;
    let mut tcp: Option<TcpStream> = None;
    let mut midi: Option<midir::MidiInputConnection<()>> = None;
    let mut parser = MidiStream::default();
    let mut decoder = SceneDecoder::default();
    let mut next_attempt = Instant::now();
    let mut next_midi_check = Instant::now();
    let mut last_published = String::new();
    loop {
        let (s, exit) = {
            let c = control.lock().unwrap();
            (c.snapshot.clone(), c.exit)
        };
        if exit {
            return;
        }
        let published = format!("{}:{}:{}:{}", s.revision, s.requested, s.status, s.detail);
        if published != last_published {
            if let Some(app) = &app {
                super::refresh_menu_bar(app);
                let _ = app.emit("bridge-status", &s);
            }
            last_published = published;
        }
        if revision != s.revision {
            tcp = None;
            midi = None;
            parser = MidiStream::default();
            revision = s.revision;
            next_attempt = Instant::now();
        }
        if !s.requested {
            thread::sleep(Duration::from_millis(50));
            continue;
        }
        let config = &s.config;
        if config.transport == "midi" && midi.is_some() && Instant::now() >= next_midi_check {
            next_midi_check = Instant::now() + Duration::from_secs(2);
            match midi_inputs() {
                Ok(inputs) if inputs.contains(&config.midi_input) => {
                    report(&control, revision, "connected", format!("MIDI input open: {}. Port presence does not confirm a desk connection.", config.midi_input));
                }
                Ok(_) => {
                    midi = None;
                    {
                        let mut c = control.lock().unwrap();
                        if c.snapshot.revision == revision && c.session == Some(session) {
                            c.session = None;
                            c.tcp_writer = None;
                        }
                    }
                    next_attempt = Instant::now();
                }
                Err(error) => report(&control, revision, "error", error),
            }
        }
        if tcp.is_none() && midi.is_none() && Instant::now() >= next_attempt {
            session += 1;
            {
                let mut c = control.lock().unwrap();
                if c.snapshot.revision == revision {
                    c.session = Some(session);
                }
            }
            decoder = SceneDecoder::default();
            report(
                &control,
                revision,
                "connecting",
                "Connecting to selected scene source".into(),
            );
            let result = if config.transport == "tcp" {
                config
                    .host
                    .parse::<IpAddr>()
                    .map_err(|_| "Enter a MixRack IP address".to_string())
                    .and_then(|ip| {
                        TcpStream::connect_timeout(
                            &SocketAddr::new(ip, config.port),
                            Duration::from_millis(500),
                        )
                        .map_err(|e| e.to_string())
                    })
                    .and_then(|stream| {
                        stream
                            .set_read_timeout(Some(Duration::from_millis(100)))
                            .map_err(|e| e.to_string())?;
                        socket2::SockRef::from(&stream)
                            .set_tcp_keepalive(
                                &socket2::TcpKeepalive::new()
                                    .with_time(Duration::from_secs(15))
                                    .with_interval(Duration::from_secs(5)),
                            )
                            .map_err(|e| e.to_string())?;
                        stream
                            .set_write_timeout(Some(Duration::from_millis(100)))
                            .map_err(|e| e.to_string())?;
                        let writer = stream.try_clone().map_err(|e| e.to_string())?;
                        {
                            let mut c = control.lock().unwrap();
                            if c.exit || c.snapshot.revision != revision || !c.snapshot.requested {
                                return Err("Connection attempt superseded".into());
                            }
                            c.tcp_writer = Some(writer);
                        }
                        tcp = Some(stream);
                        parser = MidiStream::default();
                        Ok(())
                    })
            } else {
                connect_midi(
                    &config.midi_input,
                    revision,
                    session,
                    config.channel,
                    tx.clone(),
                    app.clone(),
                )
                .map(|connection| {
                    midi = Some(connection);
                    next_midi_check = Instant::now() + Duration::from_secs(2);
                })
            };
            match result {
                Ok(()) => report(
                    &control,
                    revision,
                    "connected",
                    if config.transport == "tcp" {
                        format!("MixRack TCP open: {}:{}; MIDI channel {}. REAPER marker delivery is separate.", config.host, config.port, config.channel)
                    } else {
                        format!("MIDI input open: {}. Port presence does not confirm a desk connection.", config.midi_input)
                    },
                ),
                Err(error) => report(&control, revision, "waiting", error),
            }
            next_attempt = Instant::now() + Duration::from_secs(2);
        }
        if let Some(stream) = tcp.as_mut() {
            let mut bytes = [0; 4096];
            match stream.read(&mut bytes) {
                Ok(0) => {
                    tcp = None;
                    {
                        let mut c = control.lock().unwrap();
                        if c.snapshot.revision == revision && c.session == Some(session) {
                            c.session = None;
                            c.tcp_writer = None;
                        }
                    }
                    report(
                        &control,
                        revision,
                        "waiting",
                        "MixRack closed the connection".into(),
                    );
                }
                Ok(length) => {
                    for message in parser.feed(&bytes[..length]) {
                        enqueue(
                            &tx,
                            revision,
                            session,
                            config.channel,
                            &message,
                            &mut decoder,
                            &app,
                        );
                    }
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock
                            | std::io::ErrorKind::TimedOut
                            | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(e) => {
                    tcp = None;
                    {
                        let mut c = control.lock().unwrap();
                        if c.snapshot.revision == revision && c.session == Some(session) {
                            c.session = None;
                            c.tcp_writer = None;
                        }
                    }
                    report(&control, revision, "waiting", e.to_string());
                }
            }
        } else {
            thread::sleep(Duration::from_millis(50));
        }
    }
}
pub fn midi_inputs() -> Result<Vec<String>, String> {
    let input = midir::MidiInput::new("dLive REAPER Link").map_err(|e| e.to_string())?;
    input
        .ports()
        .iter()
        .map(|port| input.port_name(port).map_err(|e| e.to_string()))
        .collect()
}
#[derive(Clone, Debug, PartialEq)]
pub struct SceneRecall {
    pub received_at: Instant,
    pub scene_number: u16,
    pub bank: u8,
    pub lsb: u8,
    pub program: u8,
    pub channel: u8,
}
#[derive(Default)]
struct SceneDecoder {
    bank: u8,
    lsb: u8,
}
impl SceneDecoder {
    fn receive(&mut self, message: &[u8], channel: u8) -> Option<SceneRecall> {
        match message {
            [status, 0, value] if *status == 0xb0 | (channel - 1) => self.bank = *value,
            [status, 32, value] if *status == 0xb0 | (channel - 1) => self.lsb = *value,
            [status, program] if *status == 0xc0 | (channel - 1) => {
                let scene_number = super::decode_scene_number(self.bank, *program)?;
                return Some(SceneRecall {
                    received_at: Instant::now(),
                    scene_number,
                    bank: self.bank,
                    lsb: self.lsb,
                    program: *program,
                    channel,
                });
            }
            _ => {}
        }
        None
    }
}
fn enqueue(
    tx: &mpsc::SyncSender<(u64, u64, SceneRecall)>,
    revision: u64,
    session: u64,
    channel: u8,
    message: &[u8],
    decoder: &mut SceneDecoder,
    app: &Option<tauri::AppHandle>,
) {
    if let Some(app) = app {
        let _ = app.emit(
            "midi-message",
            message
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    // Decode before queuing: an overloaded marker consumer must never lose a
    // bank selection and subsequently name a scene from the wrong bank.
    if let Some(recall) = decoder.receive(message, channel) {
        if tx.try_send((revision, session, recall)).is_err() {
            if let Some(app) = app {
                let _ = app.emit(
                    "marker-decision",
                    "Scene input overloaded; scene recall discarded",
                );
            }
        }
    }
}
fn connect_midi(
    name: &str,
    revision: u64,
    session: u64,
    channel: u8,
    tx: mpsc::SyncSender<(u64, u64, SceneRecall)>,
    app: Option<tauri::AppHandle>,
) -> Result<midir::MidiInputConnection<()>, String> {
    let input = midir::MidiInput::new("dLive REAPER Link").map_err(|e| e.to_string())?;
    let port = input
        .ports()
        .into_iter()
        .find(|p| input.port_name(p).ok().as_deref() == Some(name))
        .ok_or_else(|| format!("MIDI input unavailable: {name}"))?;
    let mut decoder = SceneDecoder::default();
    input
        .connect(
            &port,
            "dLive REAPER Link",
            move |_, message, _| {
                enqueue(&tx, revision, session, channel, message, &mut decoder, &app)
            },
            (),
        )
        .map_err(|e| e.to_string())
}

/// Discard unsupported messages while still consuming their framing. Real-time
/// bytes may appear anywhere and must not disturb a partial channel message.
#[derive(Default)]
struct MidiStream {
    status: Option<u8>,
    data: Vec<u8>,
    sysex: bool,
}
impl MidiStream {
    fn feed(&mut self, bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut messages = Vec::new();
        for &byte in bytes {
            if byte >= 0xf8 {
                continue;
            }
            if byte >= 0x80 {
                self.data.clear();
                self.sysex = byte == 0xf0;
                self.status = match byte {
                    0x80..=0xef | 0xf1..=0xf3 => Some(byte),
                    _ => None,
                };
                continue;
            }
            if self.sysex {
                continue;
            }
            let Some(status) = self.status else {
                continue;
            };
            let length = match status {
                0xc0..=0xdf | 0xf1 | 0xf3 => 1,
                _ => 2,
            };
            self.data.push(byte);
            if self.data.len() == length {
                if status < 0xf0 {
                    let mut message = vec![status];
                    message.append(&mut self.data);
                    messages.push(message);
                } else {
                    self.data.clear();
                    self.status = None;
                }
            }
        }
        messages
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outgoing_scene_bytes_match_protocol_boundaries_and_channels() {
        for channel in [1, 12] {
            let b = 0xb0 + channel - 1;
            let c = 0xc0 + channel - 1;
            for (scene, bank, program) in [
                (1, 0, 0),
                (128, 0, 127),
                (129, 1, 0),
                (256, 1, 127),
                (257, 2, 0),
                (384, 2, 127),
                (385, 3, 0),
                (500, 3, 115),
            ] {
                assert_eq!(
                    scene_recall_bytes(scene, channel),
                    Ok([b, 0, bank, c, program])
                );
            }
        }
        for channel in 1..=12 {
            for scene in 1..=500 {
                let bytes = scene_recall_bytes(scene, channel).unwrap();
                let mut decoder = SceneDecoder::default();
                assert!(decoder.receive(&bytes[..3], channel).is_none());
                assert_eq!(
                    decoder.receive(&bytes[3..], channel).unwrap().scene_number,
                    scene
                );
            }
        }
        for (scene, channel) in [(0, 1), (501, 1), (1, 0), (1, 13)] {
            assert!(scene_recall_bytes(scene, channel).is_err());
        }
    }

    fn assert_no_application_bytes(socket: &mut TcpStream) {
        socket
            .set_read_timeout(Some(Duration::from_millis(150)))
            .unwrap();
        let mut byte = [0];
        let error = socket
            .read(&mut byte)
            .expect_err("unexpected application bytes or EOF");
        assert!(matches!(
            error.kind(),
            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
        ));
    }

    #[test]
    fn simulated_mixrack_receives_exact_sends_and_rejects_stale_sessions() {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let config = Config {
            transport: "tcp".into(),
            host: "127.0.0.1".into(),
            port: listener.local_addr().unwrap().port(),
            channel: 12,
            midi_input: String::new(),
        };
        let path =
            std::env::temp_dir().join(format!("dlive-send-test-{}.json", std::process::id()));
        let bridge = Bridge::new(config.clone(), path.clone(), None, |_| {});
        wait_for(&bridge, "connected");
        let (mut socket, _) = listener.accept().unwrap();
        // Baseline reception is still receive-only until an explicit internal send.
        assert_no_application_bytes(&mut socket);
        let session = bridge.send_session().unwrap();
        for (scene, expected) in [
            (500, [0xbb, 0, 3, 0xcb, 115]),
            (129, [0xbb, 0, 1, 0xcb, 0]),
            (1, [0xbb, 0, 0, 0xcb, 0]),
        ] {
            bridge.send_scene(session, scene).unwrap();
            let mut bytes = [0; 5];
            socket.read_exact(&mut bytes).unwrap();
            assert_eq!(bytes, expected);
        }
        assert!(bridge.send_scene(session, 0).is_err());
        assert!(bridge.send_scene(session, 501).is_err());
        assert_no_application_bytes(&mut socket);
        bridge.listen(false);
        assert!(bridge.send_session().is_err());
        assert!(bridge.send_scene(session, 2).is_err());
        bridge.listen(true);
        wait_for(&bridge, "connected");
        let (mut new_socket, _) = listener.accept().unwrap();
        assert!(bridge.send_scene(session, 2).is_err());
        assert_no_application_bytes(&mut new_socket);
        let new_session = bridge.send_session().unwrap();
        drop(new_socket);
        wait_for(&bridge, "waiting");
        assert!(bridge.send_scene(new_session, 3).is_err());
        wait_for(&bridge, "connected");
        let (mut recovered, _) = listener.accept().unwrap();
        assert!(bridge.send_scene(new_session, 3).is_err());
        assert_no_application_bytes(&mut recovered);
        bridge
            .send_scene(bridge.send_session().unwrap(), 128)
            .unwrap();
        let mut bytes = [0; 5];
        recovered.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, [0xbb, 0, 0, 0xcb, 127]);
        let before_change = bridge.send_session().unwrap();
        let other = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut changed = config;
        changed.port = other.local_addr().unwrap().port();
        changed.channel = 1;
        bridge.save(changed).unwrap();
        assert!(bridge.send_scene(before_change, 3).is_err());
        wait_for(&bridge, "connected");
        let (mut changed_socket, _) = other.accept().unwrap();
        assert_no_application_bytes(&mut changed_socket);
        let changed_session = bridge.send_session().unwrap();
        bridge.send_scene(changed_session, 257).unwrap();
        changed_socket.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, [0xb0, 0, 2, 0xc0, 0]);

        // Force a write failure while the receive side is still open.
        bridge
            .control
            .lock()
            .unwrap()
            .tcp_writer
            .as_ref()
            .unwrap()
            .shutdown(Shutdown::Write)
            .unwrap();
        let error = bridge.send_scene(changed_session, 2).unwrap_err();
        assert!(error.contains("delivery unknown"));
        assert!(bridge.send_scene(changed_session, 2).is_err());
        bridge.listen(false);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn midi_source_cannot_send_scenes() {
        let config = Config {
            transport: "midi".into(),
            host: String::new(),
            port: 51325,
            channel: 12,
            midi_input: String::new(),
        };
        let bridge = Bridge::new(config, PathBuf::new(), None, |_| {});
        assert!(bridge.send_session().is_err());
    }

    #[test]
    fn split_running_status_and_interleaved_realtime() {
        let mut p = MidiStream::default();
        assert!(p.feed(&[0xbb, 0]).is_empty());
        assert_eq!(
            p.feed(&[0xf8, 3, 32, 0, 0xcb, 115, 0xfa, 0]),
            vec![
                vec![0xbb, 0, 3],
                vec![0xbb, 32, 0],
                vec![0xcb, 115],
                vec![0xcb, 0]
            ]
        );
    }
    #[test]
    fn sysex_system_common_and_status_changes_do_not_leak_data() {
        let mut p = MidiStream::default();
        assert_eq!(
            p.feed(&[0xbb, 0, 0xf0, 0, 0xcb, 5, 0xf2, 1, 2, 3, 0xf7, 4, 0xcb, 6]),
            vec![vec![0xcb, 5], vec![0xcb, 6]]
        );
        assert_eq!(
            p.feed(&[0xf0, 0, 1, 2, 0xf8, 0xf7, 5, 0xcb, 7]),
            vec![vec![0xcb, 7]]
        );
    }
    #[test]
    fn every_packet_split_produces_same_messages() {
        let bytes = [0xbb, 0, 3, 32, 0, 0xcb, 115, 0, 0xbb, 0, 1, 0xcb, 0];
        let expected = MidiStream::default().feed(&bytes);
        for split in 0..=bytes.len() {
            let mut p = MidiStream::default();
            let mut actual = p.feed(&bytes[..split]);
            actual.extend(p.feed(&bytes[split..]));
            assert_eq!(actual, expected);
        }
    }
    #[test]
    fn stale_attempt_cannot_overwrite_stop_or_new_selection() {
        let control = Mutex::new(Control {
            snapshot: Snapshot {
                config: Config {
                    transport: "midi".into(),
                    host: String::new(),
                    port: 51325,
                    channel: 12,
                    midi_input: String::new(),
                },
                requested: false,
                status: "stopped".into(),
                detail: String::new(),
                revision: 2,
            },
            exit: false,
            session: None,
            tcp_writer: None,
        });
        report(&control, 1, "connected", "old connection".into());
        assert_eq!(control.lock().unwrap().snapshot.status, "stopped");
        control.lock().unwrap().snapshot.requested = true;
        report(&control, 1, "connected", "old connection".into());
        assert_eq!(control.lock().unwrap().snapshot.status, "stopped");
        report(&control, 2, "waiting", "current attempt".into());
        assert_eq!(control.lock().unwrap().snapshot.status, "waiting");
    }
    fn wait_for(bridge: &Bridge, status: &str) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while bridge.snapshot().status != status {
            assert!(
                Instant::now() < deadline,
                "expected {status}, got {}: {}",
                bridge.snapshot().status,
                bridge.snapshot().detail
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
    #[test]
    fn tcp_missing_arrival_disconnect_stop_rearm_and_selection_change() {
        use std::{io::Write, net::TcpListener};
        let reserved = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = reserved.local_addr().unwrap().port();
        drop(reserved);
        let path =
            std::env::temp_dir().join(format!("dlive-bridge-test-{}.json", std::process::id()));
        let config = Config {
            transport: "tcp".into(),
            host: "127.0.0.1".into(),
            port,
            channel: 1,
            midi_input: String::new(),
        };
        let (tx, rx) = mpsc::channel();
        let bridge = Bridge::new(config.clone(), path.clone(), None, move |scene| {
            tx.send(scene).unwrap();
        });
        wait_for(&bridge, "waiting");
        let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
        wait_for(&bridge, "connected");
        let (mut socket, _) = listener.accept().unwrap();
        socket.write_all(&[0xb0, 0]).unwrap();
        thread::sleep(Duration::from_millis(20));
        socket.write_all(&[3, 0xc0, 115]).unwrap();
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(2))
                .unwrap()
                .scene_number,
            500
        );
        drop(socket);
        wait_for(&bridge, "waiting");
        wait_for(&bridge, "connected");
        let (mut socket, _) = listener.accept().unwrap();
        socket.write_all(&[0xc0, 0]).unwrap();
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(2))
                .unwrap()
                .scene_number,
            1,
            "bank must reset on reconnect"
        );
        bridge.listen(false);
        wait_for(&bridge, "stopped");
        thread::sleep(Duration::from_millis(200));
        assert_eq!(bridge.snapshot().status, "stopped");
        assert!(!bridge.snapshot().requested);
        bridge.reconnect();
        assert!(
            !bridge.snapshot().requested,
            "Reconnect must not rearm deliberate Stop"
        );
        bridge.listen(true);
        wait_for(&bridge, "connected");
        let (socket2, _) = listener.accept().unwrap();
        let other = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut changed = config;
        changed.port = other.local_addr().unwrap().port();
        bridge.save(changed).unwrap();
        wait_for(&bridge, "connected");
        let (mut socket3, _) = other.accept().unwrap();
        socket3.write_all(&[0xc0, 127]).unwrap();
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(2))
                .unwrap()
                .scene_number,
            128
        );
        bridge.listen(false);
        drop(socket2);
        drop(socket3);
        drop(bridge);
        let _ = fs::remove_file(path);
    }
    #[test]
    fn scene_decoder_ignores_other_channels_and_invalid_banks() {
        let mut decoder = SceneDecoder::default();
        assert!(decoder.receive(&[0xb1, 0, 3], 1).is_none());
        assert_eq!(decoder.receive(&[0xc0, 0], 1).unwrap().scene_number, 1);
        decoder.receive(&[0xb0, 0, 3], 1);
        assert_eq!(decoder.receive(&[0xc0, 115], 1).unwrap().scene_number, 500);
        assert!(decoder.receive(&[0xc0, 116], 1).is_none());
        decoder.receive(&[0xb0, 0, 4], 1);
        assert!(decoder.receive(&[0xc0, 0], 1).is_none());
    }
    #[test]
    fn queued_recalls_from_lost_session_are_discarded() {
        use std::{io::Write, net::TcpListener};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let config = Config {
            transport: "tcp".into(),
            host: "127.0.0.1".into(),
            port: listener.local_addr().unwrap().port(),
            channel: 12,
            midi_input: String::new(),
        };
        let (seen_tx, seen_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let bridge = Bridge::new(config, PathBuf::new(), None, move |scene| {
            seen_tx.send(scene.scene_number).unwrap();
            release_rx.recv().unwrap();
        });
        wait_for(&bridge, "connected");
        let (mut socket, _) = listener.accept().unwrap();
        socket.write_all(&[0xcb, 0, 1]).unwrap();
        assert_eq!(seen_rx.recv_timeout(Duration::from_secs(1)).unwrap(), 1);
        drop(socket);
        wait_for(&bridge, "waiting");
        release_tx.send(()).unwrap();
        assert!(
            seen_rx.recv_timeout(Duration::from_millis(200)).is_err(),
            "queued recall must not survive lost session"
        );
        bridge.listen(false);
    }
    #[test]
    fn saved_midi_migrates_and_invalid_settings_do_not_change_connection() {
        let dir = std::env::temp_dir().join(format!("dlive-settings-test-{}", std::process::id()));
        let path = dir.join("bridge.json");
        let config = Bridge::load(path.clone(), "Saved MIDI".into()).unwrap();
        assert_eq!(config.midi_input, "Saved MIDI");
        assert_eq!(config.channel, 12);
        let bridge = Bridge::new(config.clone(), path.clone(), None, |_| {});
        bridge.listen(false);
        let mut invalid = config.clone();
        invalid.transport = "tcp".into();
        invalid.host = "not an IP".into();
        assert!(bridge.save(invalid).is_err());
        for port in [51327, 51328, 51329] {
            let mut unsupported = config.clone();
            unsupported.transport = "tcp".into();
            unsupported.host = "127.0.0.1".into();
            unsupported.port = port;
            assert!(
                bridge.save(unsupported).is_err(),
                "Surface/TLS port {port} must not become a marker source"
            );
        }
        assert_eq!(bridge.snapshot().config, config);
        bridge.save(config.clone()).unwrap();
        assert!(!bridge.snapshot().requested);
        assert_eq!(
            Bridge::load(path.clone(), "Other MIDI".into()).unwrap(),
            config
        );
        let _ = fs::remove_file(path);
        let _ = fs::remove_dir(dir);
    }
}
