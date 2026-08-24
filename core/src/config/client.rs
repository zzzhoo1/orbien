use super::server::QuicOptions;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    #[serde(rename = "serverAddr", alias = "server_addr")]
    pub server_addr: String,
    #[serde(rename = "serverPort", alias = "server_port")]
    pub server_port: u16,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub transport: TransportConfig,
    #[serde(default)]
    pub proxies: Vec<ProxyConfig>,

    #[serde(
        default = "default_udp_packet_size",
        rename = "udpPacketSize",
        alias = "udp_packet_size"
    )]
    pub udp_packet_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default = "default_auth_method")]
    pub method: String,
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(default = "default_protocol")]
    pub protocol: String,

    #[serde(
        default = "default_pool_count",
        rename = "poolCount",
        alias = "pool_count"
    )]
    pub pool_count: i32,

    #[serde(default = "default_tcp_mux", rename = "tcpMux", alias = "tcp_mux")]
    pub tcp_mux: bool,

    #[serde(
        default = "default_tcp_mux_keepalive",
        rename = "tcpMuxKeepaliveInterval",
        alias = "tcp_mux_keepalive_interval"
    )]
    pub tcp_mux_keepalive_interval: i64,

    #[serde(
        default = "default_heartbeat_interval",
        rename = "heartbeatInterval",
        alias = "heartbeat_interval"
    )]
    pub heartbeat_interval: i64,
    #[serde(
        default = "default_heartbeat_timeout",
        rename = "heartbeatTimeout",
        alias = "heartbeat_timeout"
    )]
    pub heartbeat_timeout: i64,
    #[serde(default)]
    pub quic: QuicOptions,

    #[serde(default)]
    pub tls: ClientTlsConfig,

    /// Override the per-connection yamux stream concurrency limit.
    /// Defaults to [`orbien_core::transport::MAX_NUM_STREAMS`] (256) when absent.
    /// Set via config key `transport.maxYamuxStreams` / `transport.max_yamux_streams`.
    #[serde(
        default,
        rename = "maxYamuxStreams",
        alias = "max_yamux_streams",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_yamux_streams: Option<usize>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            protocol: default_protocol(),
            pool_count: default_pool_count(),
            tcp_mux: default_tcp_mux(),
            tcp_mux_keepalive_interval: default_tcp_mux_keepalive(),
            heartbeat_interval: default_heartbeat_interval(),
            heartbeat_timeout: default_heartbeat_timeout(),
            quic: QuicOptions::default(),
            tls: ClientTlsConfig::default(),
            max_yamux_streams: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTlsConfig {
    #[serde(default = "default_tls_enable")]
    pub enable: bool,
    #[serde(default, rename = "certFile", alias = "cert_file")]
    pub cert_file: String,
    #[serde(default, rename = "keyFile", alias = "key_file")]
    pub key_file: String,
    #[serde(default, rename = "trustedCaFile", alias = "trusted_ca_file")]
    pub trusted_ca_file: String,

    #[serde(default, rename = "serverName", alias = "server_name")]
    pub server_name: String,

    #[serde(
        default = "default_disable_custom_tls_first_byte",
        rename = "disableCustomTLSFirstByte",
        alias = "disable_custom_tls_first_byte"
    )]
    pub disable_custom_tls_first_byte: bool,
}

impl Default for ClientTlsConfig {
    fn default() -> Self {
        Self {
            enable: default_tls_enable(),
            cert_file: String::new(),
            key_file: String::new(),
            trusted_ca_file: String::new(),
            server_name: String::new(),
            disable_custom_tls_first_byte: default_disable_custom_tls_first_byte(),
        }
    }
}

fn default_tls_enable() -> bool {
    true
}

fn default_disable_custom_tls_first_byte() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub name: String,
    #[serde(rename = "type", alias = "proxy_type")]
    pub proxy_type: String,
    #[serde(default = "default_local_ip", rename = "localIP", alias = "local_ip")]
    pub local_ip: String,

    #[serde(default, rename = "localPort", alias = "local_port")]
    pub local_port: u16,

    #[serde(default, rename = "remotePort", alias = "remote_port")]
    pub remote_port: u16,

    #[serde(default, rename = "customDomains", alias = "custom_domains")]
    pub custom_domains: Vec<String>,

    #[serde(default)]
    pub subdomain: String,

    #[serde(default)]
    pub locations: Vec<String>,
    #[serde(default, rename = "httpUser", alias = "http_user")]
    pub http_user: String,
    #[serde(default, rename = "httpPassword", alias = "http_password")]
    pub http_password: String,
    #[serde(default, rename = "hostHeaderRewrite", alias = "host_header_rewrite")]
    pub host_header_rewrite: String,
    #[serde(default, rename = "routeByHTTPUser", alias = "route_by_http_user")]
    pub route_by_http_user: String,

    #[serde(default)]
    pub transport: ProxyTransportConfig,

    #[serde(default)]
    pub plugin: Option<PluginConfig>,

    /// Maximum simultaneous connections allowed for this proxy.
    /// 0 (default) means unlimited.
    /// TCP/HTTP/HTTPS count live streams; UDP counts recent visitor sessions.
    #[serde(default, rename = "maxConnections", alias = "max_connections")]
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    #[serde(rename = "type", alias = "plugin_type")]
    pub plugin_type: String,

    #[serde(default, rename = "localAddr", alias = "local_addr")]
    pub local_addr: String,
    #[serde(default, rename = "crtPath", alias = "crt_path")]
    pub crt_path: String,
    #[serde(default, rename = "keyPath", alias = "key_path")]
    pub key_path: String,
    #[serde(default, rename = "hostHeaderRewrite", alias = "host_header_rewrite")]
    pub host_header_rewrite: String,

    #[serde(default, rename = "requestHeaders", alias = "request_headers")]
    pub request_headers: PluginRequestHeaders,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginRequestHeaders {
    #[serde(default)]
    pub set: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyTransportConfig {
    #[serde(default, rename = "bandwidthLimit", alias = "bandwidth_limit")]
    pub bandwidth_limit: String,

    #[serde(
        default = "default_bandwidth_limit_mode",
        rename = "bandwidthLimitMode",
        alias = "bandwidth_limit_mode"
    )]
    pub bandwidth_limit_mode: String,

    #[serde(
        default,
        rename = "proxyProtocolVersion",
        alias = "proxy_protocol_version"
    )]
    pub proxy_protocol_version: String,
}

fn default_bandwidth_limit_mode() -> String {
    "client".into()
}

fn default_auth_method() -> String {
    "token".into()
}

fn default_protocol() -> String {
    "tcp".into()
}

fn default_pool_count() -> i32 {
    1
}

fn default_tcp_mux() -> bool {
    true
}

fn default_tcp_mux_keepalive() -> i64 {
    30
}

fn default_heartbeat_interval() -> i64 {
    -1
}

fn default_heartbeat_timeout() -> i64 {
    -1
}

fn default_local_ip() -> String {
    "127.0.0.1".into()
}

fn default_udp_packet_size() -> usize {
    1500
}

impl ClientConfig {
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
    }

    pub fn complete(&mut self) {
        if !self.transport.tcp_mux {
            if self.transport.heartbeat_interval < 0 {
                self.transport.heartbeat_interval = 30;
            }
            if self.transport.heartbeat_timeout < 0 {
                self.transport.heartbeat_timeout = 90;
            }
        }

        if self.transport.tls.server_name.trim().is_empty() {
            self.transport.tls.server_name = self.server_addr.clone();
        }
    }

    pub fn tls_server_name(&self) -> &str {
        if self.transport.tls.server_name.trim().is_empty() {
            &self.server_addr
        } else {
            &self.transport.tls.server_name
        }
    }

    pub fn server_endpoint(&self) -> String {
        format!("{}:{}", self.server_addr, self.server_port)
    }

    pub fn protocol(&self) -> anyhow::Result<crate::transport::Protocol> {
        crate::transport::Protocol::parse(&self.transport.protocol).ok_or_else(|| {
            anyhow::anyhow!(
                "unsupported transport.protocol {:?}, use tcp|quic|websocket|kcp",
                self.transport.protocol
            )
        })
    }

    pub fn uses_yamux(&self) -> bool {
        self.transport.tcp_mux
            && matches!(
                self.protocol().ok(),
                Some(
                    crate::transport::Protocol::Tcp
                        | crate::transport::Protocol::Websocket
                        | crate::transport::Protocol::Kcp
                )
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ClientConfig {
        ClientConfig {
            server_addr: "example.com".into(),
            server_port: 9527,
            user: "alice".into(),
            auth: AuthConfig {
                method: "token".into(),
                token: "secret".into(),
            },
            transport: TransportConfig::default(),
            proxies: vec![],
            udp_packet_size: 1500,
        }
    }

    #[test]
    fn server_endpoint_formats() {
        let cfg = sample();
        assert_eq!(cfg.server_endpoint(), "example.com:9527");
    }

    #[test]
    fn tls_server_name_falls_back_to_server_addr() {
        let mut cfg = sample();
        cfg.transport.tls.server_name = String::new();
        assert_eq!(cfg.tls_server_name(), "example.com");
    }

    #[test]
    fn tls_server_name_uses_explicit() {
        let mut cfg = sample();
        cfg.transport.tls.server_name = "sni.example.com".into();
        assert_eq!(cfg.tls_server_name(), "sni.example.com");
    }

    #[test]
    fn complete_sets_server_name() {
        let mut cfg = sample();
        cfg.transport.tls.server_name = String::new();
        cfg.complete();
        assert_eq!(cfg.transport.tls.server_name, "example.com");
    }

    #[test]
    fn complete_repairs_negative_heartbeat_when_no_mux() {
        let mut cfg = sample();
        cfg.transport.tcp_mux = false;
        cfg.transport.heartbeat_interval = -1;
        cfg.transport.heartbeat_timeout = -1;
        cfg.complete();
        assert_eq!(cfg.transport.heartbeat_interval, 30);
        assert_eq!(cfg.transport.heartbeat_timeout, 90);
    }

    #[test]
    fn complete_keeps_heartbeat_when_mux() {
        let mut cfg = sample();
        cfg.transport.tcp_mux = true;
        cfg.transport.heartbeat_interval = -1;
        cfg.complete();
        assert_eq!(cfg.transport.heartbeat_interval, -1);
    }

    #[test]
    fn protocol_parses() {
        let mut cfg = sample();
        cfg.transport.protocol = "tcp".into();
        assert!(cfg.protocol().is_ok());
        cfg.transport.protocol = "quic".into();
        assert!(cfg.protocol().is_ok());
        cfg.transport.protocol = "websocket".into();
        assert!(cfg.protocol().is_ok());
        cfg.transport.protocol = "kcp".into();
        assert!(cfg.protocol().is_ok());
    }

    #[test]
    fn protocol_rejects_unknown() {
        let mut cfg = sample();
        cfg.transport.protocol = "sctp".into();
        assert!(cfg.protocol().is_err());
    }

    #[test]
    fn uses_yamux_only_for_muxable_protocols() {
        let mut cfg = sample();
        cfg.transport.tcp_mux = true;
        cfg.transport.protocol = "tcp".into();
        assert!(cfg.uses_yamux());
        cfg.transport.protocol = "quic".into();
        assert!(!cfg.uses_yamux());
        cfg.transport.tcp_mux = false;
        cfg.transport.protocol = "tcp".into();
        assert!(!cfg.uses_yamux());
    }

    #[test]
    fn load_parses_toml_with_defaults() {
        let dir = std::env::temp_dir();
        let path = dir.join("orbien_test_client.toml");
        std::fs::write(
            &path,
            r#"
serverAddr = "127.0.0.1"
serverPort = 9527
[[proxies]]
name = "web"
type = "http"
localIP = "127.0.0.1"
localPort = 8080
remotePort = 80
"#,
        )
        .unwrap();
        let cfg = ClientConfig::load(&path).unwrap();
        assert_eq!(cfg.server_addr, "127.0.0.1");
        assert_eq!(cfg.server_port, 9527);
        assert_eq!(cfg.proxies.len(), 1);
        assert_eq!(cfg.proxies[0].name, "web");
        assert_eq!(cfg.transport.protocol, "tcp"); // default
        // No [auth] block present -> AuthConfig::default() -> method is empty string.
        assert_eq!(cfg.auth.method, "");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_errors_on_missing_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("orbien_nonexistent_xyz.toml");
        assert!(ClientConfig::load(&path).is_err());
    }
}
