use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(
        default = "default_bind_addr",
        rename = "bindAddr",
        alias = "bind_addr"
    )]
    pub bind_addr: String,

    #[serde(
        default = "default_bind_port",
        rename = "bindPort",
        alias = "bind_port"
    )]
    pub bind_port: u16,

    #[serde(default, rename = "quicBindPort", alias = "quic_bind_port")]
    pub quic_bind_port: u16,

    #[serde(default, rename = "kcpBindPort", alias = "kcp_bind_port")]
    pub kcp_bind_port: u16,

    #[serde(default, rename = "vhostHTTPPort", alias = "vhost_http_port")]
    pub vhost_http_port: u16,

    #[serde(default, rename = "vhostHTTPSPort", alias = "vhost_https_port")]
    pub vhost_https_port: u16,

    #[serde(default, rename = "subDomainHost", alias = "sub_domain_host")]
    pub sub_domain_host: String,
    #[serde(default)]
    pub auth: AuthConfig,

    #[serde(
        default = "default_bind_addr",
        rename = "proxyBindAddr",
        alias = "proxy_bind_addr"
    )]
    pub proxy_bind_addr: String,
    #[serde(default)]
    pub transport: ServerTransportConfig,

    #[serde(default, rename = "webServer", alias = "web_server")]
    pub web_server: WebServerConfig,

    #[serde(
        default = "default_udp_packet_size",
        rename = "udpPacketSize",
        alias = "udp_packet_size"
    )]
    pub udp_packet_size: usize,

    #[serde(default, rename = "proxyProtocol", alias = "proxy_protocol")]
    pub proxy_protocol: bool,

    #[serde(
        default,
        rename = "proxyProtocolTrustedCidrs",
        alias = "proxy_protocol_trusted_cidrs"
    )]
    pub proxy_protocol_trusted_cidrs: Vec<String>,

    #[serde(default, rename = "denySrcCidrs", alias = "deny_src_cidrs")]
    pub deny_src_cidrs: Vec<String>,

    #[serde(
        default = "default_proxy_protocol_timeout",
        rename = "proxyProtocolTimeoutSecs",
        alias = "proxy_protocol_timeout_secs"
    )]
    pub proxy_protocol_timeout_secs: u64,

    /// Seconds before a UDP work-conn read is considered dead.
    /// 0 = use default (60 s).
    #[serde(
        default,
        rename = "udpWorkReadSecs",
        alias = "udp_work_read_secs"
    )]
    pub udp_work_read_secs: u64,

    /// Seconds to wait for a work-conn to arrive from the client pool.
    /// 0 = use default (10 s).
    #[serde(
        default,
        rename = "workConnTimeoutSecs",
        alias = "work_conn_timeout_secs"
    )]
    pub work_conn_timeout_secs: u64,

    /// Allow only one simultaneous control connection per user token.
    /// When a second login arrives for the same user, the old connection is
    /// kicked.  Default: false (multiple connections allowed).
    #[serde(
        default,
        rename = "singleClientPerUser",
        alias = "single_client_per_user"
    )]
    pub single_client_per_user: bool,

    /// Interval (seconds) for server-initiated Ping on the control channel.
    /// 0 = use default (30 s).
    #[serde(
        default,
        rename = "ctrlHeartbeatIntervalSecs",
        alias = "ctrl_heartbeat_interval_secs"
    )]
    pub ctrl_heartbeat_interval_secs: u64,

    /// Seconds of silence on the control channel before the connection is
    /// considered dead.  0 = use default (90 s).
    #[serde(
        default,
        rename = "ctrlHeartbeatTimeoutSecs",
        alias = "ctrl_heartbeat_timeout_secs"
    )]
    pub ctrl_heartbeat_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenPolicy {
    pub token: String,
    #[serde(default)]
    pub allowed_tunnels: Vec<String>,
    #[serde(default)]
    pub allowed_protocols: Vec<String>,
    #[serde(default)]
    pub allowed_remote_ports: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default = "default_auth_method")]
    pub method: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub token_policies: Vec<TokenPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTransportConfig {
    #[serde(default = "default_tcp_mux", rename = "tcpMux", alias = "tcp_mux")]
    pub tcp_mux: bool,

    #[serde(
        default = "default_tcp_mux_keepalive",
        rename = "tcpMuxKeepaliveInterval",
        alias = "tcp_mux_keepalive_interval"
    )]
    pub tcp_mux_keepalive_interval: i64,

    #[serde(default, rename = "maxPoolCount", alias = "max_pool_count")]
    pub max_pool_count: i64,

    #[serde(default, rename = "heartbeatTimeout", alias = "heartbeat_timeout")]
    pub heartbeat_timeout: i64,
    #[serde(default)]
    pub quic: QuicOptions,

    #[serde(default)]
    pub tls: ServerTlsConfig,
}

impl Default for ServerTransportConfig {
    fn default() -> Self {
        Self {
            tcp_mux: default_tcp_mux(),
            tcp_mux_keepalive_interval: default_tcp_mux_keepalive(),
            max_pool_count: 0,
            heartbeat_timeout: 0,
            quic: QuicOptions::default(),
            tls: ServerTlsConfig::default(),
        }
    }
}

/// Dashboard web server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebServerConfig {
    #[serde(default)]
    pub addr: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default, rename = "assetsDir", alias = "assets_dir")]
    pub assets_dir: String,
    #[serde(default, rename = "webauthnRpId", alias = "webauthn_rp_id")]
    pub webauthn_rp_id: String,
    #[serde(default, rename = "webauthnOrigin", alias = "webauthn_origin")]
    pub webauthn_origin: String,
}

impl WebServerConfig {
    pub fn complete(&mut self) {
        if self.addr.trim().is_empty() {
            self.addr = "127.0.0.1".into();
        }
    }
    pub fn enabled(&self) -> bool {
        self.port > 0
    }
    pub fn webauthn_enabled(&self) -> bool {
        !self.webauthn_rp_id.trim().is_empty() && !self.webauthn_origin.trim().is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerTlsConfig {
    #[serde(default)]
    pub force: bool,
    #[serde(default, rename = "certFile", alias = "cert_file")]
    pub cert_file: String,
    #[serde(default, rename = "keyFile", alias = "key_file")]
    pub key_file: String,
    #[serde(default, rename = "trustedCaFile", alias = "trusted_ca_file")]
    pub trusted_ca_file: String,
}

fn default_tcp_mux() -> bool {
    true
}
fn default_tcp_mux_keepalive() -> i64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicOptions {
    #[serde(
        default = "default_quic_keepalive",
        rename = "keepalivePeriod",
        alias = "keepalive_period"
    )]
    pub keepalive_period: u64,
    #[serde(
        default = "default_quic_idle",
        rename = "maxIdleTimeout",
        alias = "max_idle_timeout"
    )]
    pub max_idle_timeout: u64,
    #[serde(
        default = "default_quic_streams",
        rename = "maxIncomingStreams",
        alias = "max_incoming_streams"
    )]
    pub max_incoming_streams: u32,
}

impl Default for QuicOptions {
    fn default() -> Self {
        Self {
            keepalive_period: default_quic_keepalive(),
            max_idle_timeout: default_quic_idle(),
            max_incoming_streams: default_quic_streams(),
        }
    }
}

impl QuicOptions {
    pub fn keepalive(&self) -> Duration {
        Duration::from_secs(self.keepalive_period.max(1))
    }
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.max_idle_timeout.max(1))
    }
}

fn default_bind_addr() -> String {
    "0.0.0.0".into()
}
fn default_bind_port() -> u16 {
    9527
}
fn default_auth_method() -> String {
    "token".into()
}
fn default_quic_keepalive() -> u64 {
    10
}
fn default_quic_idle() -> u64 {
    30
}
fn default_quic_streams() -> u32 {
    100_000
}
fn default_udp_packet_size() -> usize {
    1500
}
fn default_proxy_protocol_timeout() -> u64 {
    5
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: default_bind_addr(),
            bind_port: default_bind_port(),
            quic_bind_port: 0,
            kcp_bind_port: 0,
            vhost_http_port: 0,
            vhost_https_port: 0,
            sub_domain_host: String::new(),
            auth: AuthConfig::default(),
            proxy_bind_addr: default_bind_addr(),
            transport: ServerTransportConfig::default(),
            web_server: WebServerConfig::default(),
            udp_packet_size: default_udp_packet_size(),
            proxy_protocol: false,
            proxy_protocol_trusted_cidrs: Vec::new(),
            deny_src_cidrs: Vec::new(),
            proxy_protocol_timeout_secs: default_proxy_protocol_timeout(),
            udp_work_read_secs: 0,
            work_conn_timeout_secs: 0,
            single_client_per_user: false,
            ctrl_heartbeat_interval_secs: 0,
            ctrl_heartbeat_timeout_secs: 0,
        }
    }
}

impl ServerConfig {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let raw = super::read_toml_file(path)?;
        let mut cfg: Self = toml::from_str(&raw)
            .with_context(|| format!("failed to parse config file '{}'", path.display()))?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        cfg.resolve_paths(base);
        cfg.complete();
        Ok(cfg)
    }

    fn resolve_paths(&mut self, base: &Path) {
        let tls = &mut self.transport.tls;
        tls.cert_file = super::resolve_maybe_relative(base, &tls.cert_file);
        tls.key_file = super::resolve_maybe_relative(base, &tls.key_file);
        tls.trusted_ca_file = super::resolve_maybe_relative(base, &tls.trusted_ca_file);
        if !self.web_server.assets_dir.trim().is_empty() {
            self.web_server.assets_dir =
                super::resolve_maybe_relative(base, &self.web_server.assets_dir);
        }
    }

    pub fn from_defaults() -> Self {
        let mut cfg = Self::default();
        cfg.complete();
        cfg
    }

    pub fn complete(&mut self) {
        if self.bind_addr.trim().is_empty() {
            self.bind_addr = default_bind_addr();
        }
        if self.bind_port == 0 {
            self.bind_port = default_bind_port();
        }
        if self.proxy_bind_addr.trim().is_empty() {
            self.proxy_bind_addr = self.bind_addr.clone();
        }
        if self.auth.method.trim().is_empty() {
            self.auth.method = default_auth_method();
        }
        if self.udp_packet_size == 0 {
            self.udp_packet_size = default_udp_packet_size();
        }
        self.web_server.complete();
        if self.transport.max_pool_count == 0 {
            self.transport.max_pool_count = 5;
        }
        if self.transport.heartbeat_timeout == 0 {
            self.transport.heartbeat_timeout = if self.transport.tcp_mux { -1 } else { 90 };
        }
        if !self.transport.tls.trusted_ca_file.trim().is_empty() {
            self.transport.tls.force = true;
        }
        // Fill in configurable-timeout defaults
        if self.udp_work_read_secs == 0 {
            self.udp_work_read_secs = 60;
        }
        if self.work_conn_timeout_secs == 0 {
            self.work_conn_timeout_secs = 10;
        }
        if self.ctrl_heartbeat_interval_secs == 0 {
            self.ctrl_heartbeat_interval_secs = 30;
        }
        if self.ctrl_heartbeat_timeout_secs == 0 {
            self.ctrl_heartbeat_timeout_secs = 90;
        }
    }

    pub fn quic_enabled(&self) -> bool {
        self.quic_bind_port != 0
    }
    pub fn kcp_enabled(&self) -> bool {
        self.kcp_bind_port != 0
    }
    pub fn vhost_http_enabled(&self) -> bool {
        self.vhost_http_port != 0
    }
    pub fn vhost_https_enabled(&self) -> bool {
        self.vhost_https_port != 0
    }

    /// Effective UDP work-conn read deadline.
    pub fn udp_work_read_deadline(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.udp_work_read_secs.max(1))
    }

    /// Effective work-conn wait timeout.
    pub fn work_conn_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.work_conn_timeout_secs.max(1))
    }

    /// Effective control-channel heartbeat interval.
    pub fn ctrl_heartbeat_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.ctrl_heartbeat_interval_secs.max(1))
    }

    /// Effective control-channel heartbeat timeout.
    pub fn ctrl_heartbeat_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.ctrl_heartbeat_timeout_secs.max(1))
    }
}
