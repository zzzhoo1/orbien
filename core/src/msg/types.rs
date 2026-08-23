use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};

pub const TYPE_LOGIN: u8 = b'o';
pub const TYPE_LOGIN_RESP: u8 = b'1';
pub const TYPE_NEW_PROXY: u8 = b'p';
pub const TYPE_NEW_PROXY_RESP: u8 = b'2';
pub const TYPE_CLOSE_PROXY: u8 = b'c';
pub const TYPE_NEW_WORK_CONN: u8 = b'w';
pub const TYPE_REQ_WORK_CONN: u8 = b'r';
pub const TYPE_START_WORK_CONN: u8 = b's';
pub const TYPE_PING: u8 = b'h';
pub const TYPE_PONG: u8 = b'4';

pub const TYPE_UDP_PACKET: u8 = b'u';
pub const TYPE_KICK_OUT: u8 = b'k';

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Login {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub hostname: String,
    #[serde(default)]
    pub os: String,
    #[serde(default)]
    pub arch: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub privilege_key: String,
    #[serde(default)]
    pub timestamp: i64,
    #[serde(default)]
    pub run_id: String,
    #[serde(default)]
    pub pool_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResp {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub run_id: String,
    #[serde(default)]
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProxy {
    pub proxy_name: String,
    pub proxy_type: String,

    #[serde(default)]
    pub remote_port: i32,

    #[serde(default)]
    pub local_ip: String,
    #[serde(default)]
    pub local_port: i32,

    #[serde(default)]
    pub custom_domains: Vec<String>,
    #[serde(default)]
    pub subdomain: String,
    #[serde(default)]
    pub locations: Vec<String>,
    #[serde(default)]
    pub http_user: String,
    #[serde(default)]
    pub http_pwd: String,
    #[serde(default)]
    pub host_header_rewrite: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub response_headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub route_by_http_user: String,

    #[serde(default)]
    pub bandwidth_limit: String,

    #[serde(default)]
    pub bandwidth_limit_mode: String,

    /// Maximum simultaneous connections for this proxy.
    /// 0 means unlimited (default).
    #[serde(default, rename = "maxConnections")]
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProxyResp {
    pub proxy_name: String,
    #[serde(default)]
    pub remote_addr: String,
    #[serde(default)]
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloseProxy {
    pub proxy_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReqWorkConn {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWorkConn {
    pub run_id: String,
    #[serde(default)]
    pub privilege_key: String,
    #[serde(default)]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkConn {
    pub proxy_name: String,
    #[serde(default)]
    pub src_addr: String,
    #[serde(default)]
    pub src_port: u16,
    #[serde(default)]
    pub dst_addr: String,
    #[serde(default)]
    pub dst_port: u16,
    #[serde(default)]
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ping {
    #[serde(default)]
    pub privilege_key: String,
    #[serde(default)]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Pong {
    #[serde(default)]
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KickOut {
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UdpSocketAddr {
    #[serde(rename = "IP")]
    pub ip: String,
    #[serde(rename = "Port")]
    pub port: u16,
    #[serde(rename = "Zone", default, skip_serializing_if = "String::is_empty")]
    pub zone: String,
}

impl UdpSocketAddr {
    pub fn from_std(addr: SocketAddr) -> Self {
        Self {
            ip: addr.ip().to_string(),
            port: addr.port(),
            zone: String::new(),
        }
    }

    pub fn to_std(&self) -> Option<SocketAddr> {
        let ip: IpAddr = self.ip.parse().ok()?;
        Some(SocketAddr::new(ip, self.port))
    }

    pub fn key(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpPacket {
    #[serde(
        rename = "c",
        default,
        with = "b64_bytes",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub content: Vec<u8>,
    #[serde(rename = "l", default, skip_serializing_if = "Option::is_none")]
    pub local_addr: Option<UdpSocketAddr>,
    #[serde(rename = "r", default, skip_serializing_if = "Option::is_none")]
    pub remote_addr: Option<UdpSocketAddr>,
}

impl UdpPacket {
    pub fn new(content: Vec<u8>, remote: Option<SocketAddr>) -> Self {
        Self {
            content,
            local_addr: None,
            remote_addr: remote.map(UdpSocketAddr::from_std),
        }
    }
}

mod b64_bytes {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(data))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            return Ok(Vec::new());
        }
        STANDARD
            .decode(s.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Message {
    Login(Login),
    LoginResp(LoginResp),
    NewProxy(Box<NewProxy>),
    NewProxyResp(NewProxyResp),
    CloseProxy(CloseProxy),
    ReqWorkConn(ReqWorkConn),
    NewWorkConn(NewWorkConn),
    StartWorkConn(StartWorkConn),
    Ping(Ping),
    Pong(Pong),
    UdpPacket(UdpPacket),
    KickOut(KickOut),
}

impl Message {
    pub fn type_byte(&self) -> u8 {
        match self {
            Self::Login(_) => TYPE_LOGIN,
            Self::LoginResp(_) => TYPE_LOGIN_RESP,
            Self::NewProxy(_) => TYPE_NEW_PROXY,
            Self::NewProxyResp(_) => TYPE_NEW_PROXY_RESP,
            Self::CloseProxy(_) => TYPE_CLOSE_PROXY,
            Self::ReqWorkConn(_) => TYPE_REQ_WORK_CONN,
            Self::NewWorkConn(_) => TYPE_NEW_WORK_CONN,
            Self::StartWorkConn(_) => TYPE_START_WORK_CONN,
            Self::Ping(_) => TYPE_PING,
            Self::Pong(_) => TYPE_PONG,
            Self::UdpPacket(_) => TYPE_UDP_PACKET,
            Self::KickOut(_) => TYPE_KICK_OUT,
        }
    }
}
