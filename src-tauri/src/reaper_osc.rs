use rosc::{decoder, encoder, OscMessage, OscPacket, OscType};
use std::error::Error;
use std::fmt;
use std::io;
use std::net::{SocketAddr, UdpSocket};
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

impl ReaperOscClient {
    pub fn new(config: ReaperOscConfig) -> Self {
        Self { config }
    }

    pub fn create_marker(&self, name: &str) -> Result<String, ReaperOscError> {
        let socket = self.bind_feedback_socket()?;
        let reaper_addr = self.reaper_addr()?;

        send_message(
            &socket,
            reaper_addr,
            "/action",
            vec![OscType::Int(CREATE_MARKER_ACTION_ID)],
        )?;

        let marker_number = wait_for_last_marker_number(&socket, self.config.feedback_timeout)?;

        send_message(
            &socket,
            reaper_addr,
            "/lastmarker/name",
            vec![OscType::String(name.to_string())],
        )?;

        Ok(marker_number)
    }

    fn bind_feedback_socket(&self) -> Result<UdpSocket, ReaperOscError> {
        let socket = UdpSocket::bind((self.config.host.as_str(), self.config.feedback_port))
            .map_err(|source| ReaperOscError::BindFeedbackSocket {
                addr: format!("{}:{}", self.config.host, self.config.feedback_port),
                source,
            })?;

        socket
            .set_read_timeout(Some(self.config.feedback_timeout))
            .map_err(ReaperOscError::ConfigureSocket)?;

        Ok(socket)
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
                "timed out after {:.1}s waiting for /lastmarker/number/str from REAPER",
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

fn wait_for_last_marker_number(
    socket: &UdpSocket,
    timeout: Duration,
) -> Result<String, ReaperOscError> {
    let deadline = Instant::now() + timeout;
    let mut buffer = [0; 8192];

    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or(ReaperOscError::MarkerNumberTimeout { timeout })?;

        socket
            .set_read_timeout(Some(remaining))
            .map_err(ReaperOscError::ConfigureSocket)?;

        let length = match socket.recv_from(&mut buffer) {
            Ok((length, _sender)) => length,
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                return Err(ReaperOscError::MarkerNumberTimeout { timeout });
            }
            Err(source) => return Err(ReaperOscError::ReceiveOsc(source)),
        };

        let (_, packet) =
            decoder::decode_udp(&buffer[..length]).map_err(ReaperOscError::DecodeOsc)?;

        if let Some(marker_number) = last_marker_number_from_packet(&packet) {
            return Ok(marker_number);
        }
    }
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
