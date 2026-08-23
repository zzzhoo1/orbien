mod access;
mod control;
mod dashboard;
mod metrics;
mod proxy;
mod service;

use anyhow::Result;
use clap::Parser;
use orbien_core::config::ServerConfig;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "orbien-server",
    about = "orbien server — TCP tunnel",
    after_help = "Without -c/--config, orbien-server uses built-in defaults:\n  bind 0.0.0.0:9527, QUIC/KCP/vhost/dashboard disabled unless set via flags."
)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[arg(long = "bind_addr", default_value = "0.0.0.0")]
    bind_addr: String,

    #[arg(short = 'p', long = "bind_port", default_value_t = 9527)]
    bind_port: u16,

    #[arg(long = "kcp_bind_port", default_value_t = 0)]
    kcp_bind_port: u16,

    #[arg(long = "quic_bind_port", default_value_t = 0)]
    quic_bind_port: u16,

    #[arg(long = "proxy_bind_addr", default_value = "0.0.0.0")]
    proxy_bind_addr: String,

    #[arg(long = "vhost_http_port", default_value_t = 0)]
    vhost_http_port: u16,

    #[arg(long = "vhost_https_port", default_value_t = 0)]
    vhost_https_port: u16,

    #[arg(long = "dashboard_addr", default_value = "0.0.0.0")]
    dashboard_addr: String,

    #[arg(long = "dashboard_port", default_value_t = 0)]
    dashboard_port: u16,

    #[arg(long = "dashboard_user", default_value = "")]
    dashboard_user: String,

    #[arg(long = "dashboard_pwd", default_value = "")]
    dashboard_pwd: String,

    #[arg(short = 't', long = "token", default_value = "")]
    token: String,

    #[arg(long = "subdomain_host", default_value = "")]
    subdomain_host: String,

    #[arg(long = "tls_only", default_value_t = false)]
    tls_only: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let args = Args::parse();
    let cfg = load_server_config(&args)?;

    tracing::info!(
        bind = %format!("{}:{}", cfg.bind_addr, cfg.bind_port),
        quic_bind = cfg.quic_bind_port,
        kcp_bind = cfg.kcp_bind_port,
        vhost_http = cfg.vhost_http_port,
        vhost_https = cfg.vhost_https_port,
        "starting orbien-server"
    );

    service::Service::new(cfg)?.run().await
}

fn load_server_config(args: &Args) -> Result<ServerConfig> {
    if let Some(path) = args
        .config
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        tracing::info!(config = %path, "loading config");
        let cfg = ServerConfig::load(path)?;
        validate_dashboard_config(&cfg)?;
        return Ok(cfg);
    }

    tracing::info!("using CLI flags for config");
    // fix: field_reassign_with_default — use struct literal with ..Default::default()
    let mut cfg = ServerConfig {
        bind_addr: args.bind_addr.clone(),
        bind_port: args.bind_port,
        kcp_bind_port: args.kcp_bind_port,
        quic_bind_port: args.quic_bind_port,
        proxy_bind_addr: args.proxy_bind_addr.clone(),
        vhost_http_port: args.vhost_http_port,
        vhost_https_port: args.vhost_https_port,
        sub_domain_host: args.subdomain_host.clone(),
        ..ServerConfig::default()
    };
    cfg.auth.token = args.token.clone();
    cfg.web_server.addr = args.dashboard_addr.clone();
    cfg.web_server.port = args.dashboard_port;
    cfg.web_server.user = args.dashboard_user.clone();
    cfg.web_server.password = args.dashboard_pwd.clone();
    cfg.transport.tls.force = args.tls_only;
    cfg.complete();
    validate_dashboard_config(&cfg)?;
    Ok(cfg)
}

fn validate_dashboard_config(cfg: &ServerConfig) -> Result<()> {
    // If dashboard is enabled (port > 0), require explicit credentials
    if cfg.web_server.port > 0 {
        let user = cfg.web_server.user.trim();
        let password = cfg.web_server.password.trim();
        
        if user.is_empty() || password.is_empty() {
            anyhow::bail!(
                "Dashboard is enabled (port {}), but credentials are not set. \
                 Please provide --dashboard_user and --dashboard_pwd or configure \
                 them in the config file to prevent unauthorized access.",
                cfg.web_server.port
            );
        }
        
        // Warn about weak credentials
        if user == "admin" && password == "admin" {
            tracing::warn!(
                "Dashboard is using default credentials (admin/admin). \
                 This is insecure and should be changed in production environments."
            );
        }
    }
    Ok(())
}
