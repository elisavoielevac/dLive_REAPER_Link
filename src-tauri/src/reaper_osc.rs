use rosc::{decoder, encoder, OscMessage, OscPacket, OscType};
use std::error::Error;
use std::fmt;
use std::io;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_REAPER_RECEIVE_PORT: u16 = 49101;
const DEFAULT_REAPER_FEEDBACK_PORT: u16 = 49102;
const CREATE_MARKER_ACTION_ID: i32 = 40157;
const DEFAULT_FEEDBACK_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone)]
pub struct ReaperOscConfig {
    pub host: String,
    pub receive_port: u16,
    pub feedback_port: u16,
    pub feedback_timeout: Duration,
}

impl Default for ReaperOscConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            receive_port: DEFAULT_REAPER_RECEIVE_PORT,
            feedback_port: DEFAULT_REAPER_FEEDBACK_PORT,
            feedback_timeout: DEFAULT_FEEDBACK_TIMEOUT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReaperOscClient {
    config: ReaperOscConfig,
}

// Latest observed feedback only: separate OSC messages are not an atomic snapshot.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct PlaybackFeedback {
    pub playing: Option<bool>,
    pub paused: Option<bool>,
    pub stopped: Option<bool>,
    pub recording: Option<bool>,
    pub marker_name: Option<String>,
    pub marker_number: Option<String>,
    pub marker_time: Option<f32>,
    pub position: Option<f32>,
    #[serde(skip)]
    pub(crate) changed_at: Option<Instant>,
    #[serde(skip)]
    pub(crate) received_at: Option<Instant>,
}

impl PlaybackFeedback {
    fn update(&mut self, packet: &OscPacket) {
        self.update_at(packet, Instant::now());
    }

    fn update_at(&mut self, packet: &OscPacket, now: Instant) {
        match packet {
            OscPacket::Bundle(bundle) => {
                for packet in &bundle.content {
                    self.update_at(packet, now);
                }
            }
            OscPacket::Message(message) => {
                let value = message.args.first();
                let relevant = matches!(
                    message.addr.as_str(),
                    "/play"
                        | "/pause"
                        | "/stop"
                        | "/record"
                        | "/lastmarker/name"
                        | "/lastmarker/number/str"
                        | "/lastmarker/time"
                );
                if relevant {
                    self.changed_at = Some(now);
                }
                if relevant || message.addr == "/time" {
                    self.received_at = Some(now);
                }
                match message.addr.as_str() {
                    "/play" => {
                        if let Some(v) = osc_bool(value) {
                            self.playing = Some(v);
                        }
                    }
                    "/pause" => {
                        if let Some(v) = osc_bool(value) {
                            self.paused = Some(v);
                        }
                    }
                    "/stop" => {
                        if let Some(v) = osc_bool(value) {
                            self.stopped = Some(v);
                        }
                    }
                    "/record" => {
                        if let Some(v) = osc_bool(value) {
                            self.recording = Some(v);
                        }
                    }
                    "/lastmarker/name" => {
                        if let Some(OscType::String(v)) = value {
                            self.marker_name = Some(v.clone());
                        }
                    }
                    "/lastmarker/number/str" => {
                        if let Some(OscType::String(v)) = value {
                            self.marker_number = Some(v.clone());
                        }
                    }
                    "/lastmarker/time" | "/time" => {
                        if let Some(OscType::Float(v)) = value {
                            if v.is_finite() {
                                if message.addr == "/time" {
                                    if let Some(previous) = self.position {
                                        if *v < previous || (*v - previous).abs() > 0.5 {
                                            self.changed_at = Some(now);
                                        }
                                    }
                                    self.position = Some(*v);
                                } else {
                                    self.marker_time = Some(*v);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn settled(&self, now: Instant) -> bool {
        self.changed_at
            .is_some_and(|at| now.saturating_duration_since(at) >= Duration::from_millis(100))
    }

    pub fn preview_status(&self) -> &'static str {
        if self.recording == Some(true) {
            return "recording";
        }
        if self.paused == Some(true) {
            return "paused";
        }
        if self.stopped == Some(true) || self.playing == Some(false) {
            return "stopped";
        }
        if self.playing == Some(true)
            && self.recording == Some(false)
            && self.paused == Some(false)
            && self.stopped == Some(false)
        {
            return "playing";
        }
        "unknown"
    }
}

fn osc_bool(value: Option<&OscType>) -> Option<bool> {
    match value {
        Some(OscType::Bool(v)) => Some(*v),
        Some(OscType::Int(v)) => Some(*v != 0),
        Some(OscType::Float(v)) if v.is_finite() => Some(*v != 0.0),
        _ => None,
    }
}

#[derive(Clone)]
pub struct ReaperOscFeedback {
    socket: Arc<UdpSocket>,
    playback: Arc<RwLock<PlaybackFeedback>>,
    expected_host: Arc<RwLock<String>>,
    marker_feedback: Arc<(Mutex<(u64, String)>, Condvar)>,
    marker_request: Arc<Mutex<()>>,
}

impl ReaperOscFeedback {
    pub fn start(config: &ReaperOscConfig) -> Result<Self, ReaperOscError> {
        let socket = Arc::new(UdpSocket::bind(("0.0.0.0", config.feedback_port)).map_err(
            |source| ReaperOscError::BindFeedbackSocket {
                addr: format!("0.0.0.0:{}", config.feedback_port),
                source,
            },
        )?);
        socket
            .set_read_timeout(Some(Duration::from_millis(250)))
            .map_err(ReaperOscError::ConfigureSocket)?;

        let expected_host = Arc::new(RwLock::new(config.host.clone()));
        let listener_host = Arc::clone(&expected_host);
        let playback = Arc::new(RwLock::new(PlaybackFeedback::default()));
        let marker_feedback = Arc::new((Mutex::new((0, String::new())), Condvar::new()));
        let listener_socket = Arc::clone(&socket);
        let listener_playback = Arc::clone(&playback);
        let listener_marker_feedback = Arc::clone(&marker_feedback);

        thread::spawn(move || loop {
            let mut buffer = [0; 8192];
            let Ok((length, sender)) = listener_socket.recv_from(&mut buffer) else {
                continue;
            };
            // A target change cannot leave feedback from the old REAPER driving recalls.
            let host = listener_host.read().unwrap();
            if host.parse::<IpAddr>().ok() != Some(sender.ip()) {
                continue;
            }
            let Ok((_, packet)) = decoder::decode_udp(&buffer[..length]) else {
                continue;
            };

            if let Ok(mut state) = listener_playback.write() {
                state.update(&packet);
            }
            if let Some(marker_number) = last_marker_number_from_packet(&packet) {
                let (lock, changed) = &*listener_marker_feedback;
                if let Ok(mut feedback) = lock.lock() {
                    feedback.0 += 1;
                    feedback.1 = marker_number;
                    changed.notify_all();
                }
            }
        });

        Ok(Self {
            socket,
            playback,
            expected_host,
            marker_feedback,
            marker_request: Arc::new(Mutex::new(())),
        })
    }

    pub fn reset_target(&self, host: &str) {
        let mut target = self.expected_host.write().unwrap();
        *target = host.to_string();
        *self.playback.write().unwrap() = PlaybackFeedback::default();
    }

    pub fn playback(&self) -> Result<PlaybackFeedback, String> {
        self.playback
            .read()
            .map(|state| state.clone())
            .map_err(|_| "REAPER feedback is unavailable".to_string())
    }

    pub fn recording(&self) -> Option<bool> {
        self.playback.read().ok().and_then(|state| state.recording)
    }
}

impl ReaperOscClient {
    pub fn new(config: ReaperOscConfig) -> Self {
        Self { config }
    }

    pub fn create_marker(
        &self,
        feedback: &ReaperOscFeedback,
        name: &str,
    ) -> Result<String, ReaperOscError> {
        let reaper_addr = self.reaper_addr()?;
        let _request = feedback.marker_request.lock().map_err(|_| {
            ReaperOscError::ReceiveOsc(io::Error::other("marker request unavailable"))
        })?;
        let (lock, changed) = &*feedback.marker_feedback;
        let mut marker = lock.lock().map_err(|_| {
            ReaperOscError::ReceiveOsc(io::Error::other("marker feedback unavailable"))
        })?;
        let feedback_sequence = marker.0;

        send_message(
            &feedback.socket,
            reaper_addr,
            "/action",
            vec![OscType::Int(CREATE_MARKER_ACTION_ID)],
        )?;

        let deadline = Instant::now() + self.config.feedback_timeout;
        while marker.0 == feedback_sequence {
            let remaining = deadline.checked_duration_since(Instant::now()).ok_or(
                ReaperOscError::MarkerNumberTimeout {
                    timeout: self.config.feedback_timeout,
                },
            )?;
            let (next_marker, timeout) = changed.wait_timeout(marker, remaining).map_err(|_| {
                ReaperOscError::ReceiveOsc(io::Error::other("marker feedback unavailable"))
            })?;
            marker = next_marker;
            if timeout.timed_out() && marker.0 == feedback_sequence {
                return Err(ReaperOscError::MarkerNumberTimeout {
                    timeout: self.config.feedback_timeout,
                });
            }
        }
        let marker_number = marker.1.clone();
        drop(marker);

        send_message(
            &feedback.socket,
            reaper_addr,
            "/lastmarker/name",
            vec![OscType::String(name.to_string())],
        )?;

        Ok(marker_number)
    }
    fn reaper_addr(&self) -> Result<SocketAddr, ReaperOscError> {
        format!("{}:{}", self.config.host, self.config.receive_port)
            .parse()
            .map_err(|source| ReaperOscError::InvalidReaperAddress {
                addr: format!("{}:{}", self.config.host, self.config.receive_port),
                source,
            })
    }
}

impl Default for ReaperOscClient {
    fn default() -> Self {
        Self::new(ReaperOscConfig::default())
    }
}

#[derive(Debug)]
pub enum ReaperOscError {
    InvalidReaperAddress {
        addr: String,
        source: std::net::AddrParseError,
    },
    BindFeedbackSocket {
        addr: String,
        source: io::Error,
    },
    ConfigureSocket(io::Error),
    EncodeOsc(rosc::OscError),
    SendOsc {
        addr: SocketAddr,
        source: io::Error,
    },
    ReceiveOsc(io::Error),
    DecodeOsc(rosc::OscError),
    MarkerNumberTimeout {
        timeout: Duration,
    },
}

impl fmt::Display for ReaperOscError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidReaperAddress { addr, .. } => {
                write!(f, "invalid REAPER OSC address: {addr}")
            }
            Self::BindFeedbackSocket { addr, .. } => {
                write!(f, "failed to bind REAPER OSC feedback socket at {addr}")
            }
            Self::ConfigureSocket(_) => write!(f, "failed to configure REAPER OSC socket"),
            Self::EncodeOsc(_) => write!(f, "failed to encode OSC message"),
            Self::SendOsc { addr, .. } => write!(f, "failed to send OSC message to {addr}"),
            Self::ReceiveOsc(_) => write!(f, "failed to receive REAPER OSC feedback"),
            Self::DecodeOsc(_) => write!(f, "failed to decode REAPER OSC feedback"),
            Self::MarkerNumberTimeout { timeout } => write!(
                f,
                "timed out after {:.1}s waiting for REAPER OSC feedback (/lastmarker/number/str). Check that REAPER OSC feedback is enabled and sending to local port 49102",
                timeout.as_secs_f32()
            ),
        }
    }
}

impl Error for ReaperOscError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidReaperAddress { source, .. } => Some(source),
            Self::BindFeedbackSocket { source, .. } => Some(source),
            Self::ConfigureSocket(source) => Some(source),
            Self::EncodeOsc(source) => Some(source),
            Self::SendOsc { source, .. } => Some(source),
            Self::ReceiveOsc(source) => Some(source),
            Self::DecodeOsc(source) => Some(source),
            Self::MarkerNumberTimeout { .. } => None,
        }
    }
}

fn send_message(
    socket: &UdpSocket,
    addr: SocketAddr,
    osc_addr: &str,
    args: Vec<OscType>,
) -> Result<(), ReaperOscError> {
    let packet = OscPacket::Message(OscMessage {
        addr: osc_addr.to_string(),
        args,
    });
    let encoded = encoder::encode(&packet).map_err(ReaperOscError::EncodeOsc)?;

    socket
        .send_to(&encoded, addr)
        .map_err(|source| ReaperOscError::SendOsc { addr, source })?;

    Ok(())
}

fn last_marker_number_from_packet(packet: &OscPacket) -> Option<String> {
    match packet {
        OscPacket::Message(message) => last_marker_number_from_message(message),
        OscPacket::Bundle(bundle) => bundle
            .content
            .iter()
            .find_map(last_marker_number_from_packet),
    }
}

fn last_marker_number_from_message(message: &OscMessage) -> Option<String> {
    if message.addr != "/lastmarker/number/str" {
        return None;
    }

    match message.args.first() {
        Some(OscType::String(marker_number)) => Some(marker_number.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod playback_tests {
    use super::*;

    fn message(addr: &str, value: OscType) -> OscPacket {
        OscPacket::Message(OscMessage {
            addr: addr.into(),
            args: vec![value],
        })
    }

    #[test]
    fn listener_uses_configured_reaper_host_and_clears_target_state() {
        let feedback = ReaperOscFeedback::start(&ReaperOscConfig {
            host: "192.0.2.1".into(),
            feedback_port: 0,
            ..Default::default()
        })
        .unwrap();
        let address = SocketAddr::new(
            "127.0.0.1".parse().unwrap(),
            feedback.socket.local_addr().unwrap().port(),
        );
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let packet = encoder::encode(&message(
            "/lastmarker/name",
            OscType::String("Song A".into()),
        ))
        .unwrap();
        sender.send_to(&packet, address).unwrap();
        thread::sleep(Duration::from_millis(100));
        assert!(
            feedback.playback().unwrap().marker_name.is_none(),
            "wrong host ignored"
        );
        feedback.reset_target("127.0.0.1");
        sender.send_to(&packet, address).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while feedback.playback().unwrap().marker_name.is_none() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            feedback.playback().unwrap().marker_name.as_deref(),
            Some("Song A")
        );
        feedback.reset_target("192.0.2.1");
        assert!(feedback.playback().unwrap().marker_name.is_none());
    }

    #[test]
    fn startup_requires_transport_and_retains_stopped_marker() {
        let mut state = PlaybackFeedback::default();
        state.update(&message(
            "/lastmarker/name",
            OscType::String("Song A".into()),
        ));
        state.update(&message("/play", OscType::Float(1.0)));
        assert_eq!(state.preview_status(), "unknown");
        for addr in ["/record", "/pause", "/stop"] {
            state.update(&message(addr, OscType::Float(0.0)));
        }
        assert_eq!(state.preview_status(), "playing");
        assert_eq!(state.marker_name.as_deref(), Some("Song A"));
        state.update(&message("/record", OscType::Float(1.0)));
        assert_eq!(state.preview_status(), "recording");
    }

    #[test]
    fn nested_bundle_applies_all_updates_in_order() {
        let mut state = PlaybackFeedback::default();
        state.update(&OscPacket::Bundle(rosc::OscBundle {
            timetag: (0, 1).into(),
            content: vec![
                message("/record", OscType::Int(1)),
                OscPacket::Bundle(rosc::OscBundle {
                    timetag: (0, 1).into(),
                    content: vec![
                        message("/record", OscType::Int(0)),
                        message("/pause", OscType::Bool(true)),
                        message("/lastmarker/name", OscType::String("Song B".into())),
                        message("/lastmarker/number/str", OscType::String("99".into())),
                        message("/time", OscType::Float(15.0)),
                    ],
                }),
            ],
        }));
        assert_eq!(state.recording, Some(false));
        assert_eq!(state.preview_status(), "paused");
        assert_eq!(state.marker_name.as_deref(), Some("Song B"));
        assert_eq!(state.marker_number.as_deref(), Some("99"));
        assert_eq!(state.position, Some(15.0));
        state.update(&message("/time", OscType::Float(f32::NAN)));
        assert_eq!(state.position, Some(15.0));
    }
}
