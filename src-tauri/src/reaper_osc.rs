use rosc::{decoder, encoder, OscMessage, OscPacket, OscType};
use std::error::Error;
use std::fmt;
use std::io;
use std::net::{SocketAddr, UdpSocket};
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

#[derive(Clone)]
pub struct ReaperOscFeedback {
    socket: Arc<UdpSocket>,
    recording: Arc<RwLock<Option<bool>>>,
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

        let recording = Arc::new(RwLock::new(None));
        let marker_feedback = Arc::new((Mutex::new((0, String::new())), Condvar::new()));
        let listener_socket = Arc::clone(&socket);
        let listener_recording = Arc::clone(&recording);
        let listener_marker_feedback = Arc::clone(&marker_feedback);

        thread::spawn(move || loop {
            let mut buffer = [0; 8192];
            let Ok((length, _sender)) = listener_socket.recv_from(&mut buffer) else {
                continue;
            };
            let Ok((_, packet)) = decoder::decode_udp(&buffer[..length]) else {
                continue;
            };

            if let Some(recording) = recording_from_packet(&packet) {
                if let Ok(mut state) = listener_recording.write() {
                    *state = Some(recording);
                }
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
            recording,
            marker_feedback,
            marker_request: Arc::new(Mutex::new(())),
        })
    }

    pub fn recording(&self) -> Option<bool> {
        self.recording.read().ok().and_then(|state| *state)
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

fn recording_from_packet(packet: &OscPacket) -> Option<bool> {
    match packet {
        OscPacket::Message(message) if message.addr == "/record" => match message.args.first() {
            Some(OscType::Bool(value)) => Some(*value),
            Some(OscType::Int(value)) => Some(*value != 0),
            Some(OscType::Float(value)) => Some(*value != 0.0),
            _ => None,
        },
        OscPacket::Bundle(bundle) => bundle.content.iter().find_map(recording_from_packet),
        _ => None,
    }
}
