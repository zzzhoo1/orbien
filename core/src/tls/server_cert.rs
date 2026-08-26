use anyhow::{Context, Result};
use rustls::pki_types::PrivateKeyDer;
use rustls::ServerConfig;
use std::sync::Arc;

pub fn load_or_generate_https_server_config(
    cert_file: &str,
    key_file: &str,
    common_name: &str,
) -> Result<Arc<ServerConfig>> {
    crate::transport::install_ring_provider()?;

    let (certs, key) = if cert_file.trim().is_empty() || key_file.trim().is_empty() {
        tracing::info!(
            cn = %common_name,
            "plugin TLS: certFile/keyFile empty — generating ephemeral self-signed cert"
        );
        let gen = crate::transport::generate_self_signed_cert(common_name)?;
        let key = PrivateKeyDer::Pkcs8(gen.key);
        (gen.certs, key)
    } else {
        crate::transport::load_pem_cert_key(cert_file, key_file)?
    };

    let mut cfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("build https plugin ServerConfig")?;

    cfg.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(Arc::new(cfg))
}
