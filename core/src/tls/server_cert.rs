use anyhow::{bail, Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

pub fn load_or_generate_https_server_config(
    crt_path: &str,
    key_path: &str,
    common_name: &str,
) -> Result<Arc<ServerConfig>> {
    crate::transport::install_ring_provider()?;

    let (certs, key) = if crt_path.trim().is_empty() || key_path.trim().is_empty() {
        tracing::info!(
            cn = %common_name,
            "plugin TLS: crtPath/keyPath empty — generating ephemeral self-signed cert"
        );
        let gen = crate::transport::generate_self_signed_cert(common_name)?;
        let key = PrivateKeyDer::Pkcs8(gen.key);
        (gen.certs, key)
    } else {
        load_pem_cert_key(crt_path, key_path)?
    };

    let mut cfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("build https plugin ServerConfig")?;

    cfg.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(Arc::new(cfg))
}

fn load_pem_cert_key(
    crt_path: &str,
    key_path: &str,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    // Prevent path traversal attacks by rejecting paths containing '..'.
    let crt_path_obj = Path::new(crt_path);
    if crt_path_obj
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        bail!("Invalid input: {}", crt_path_obj.display());
    }
    let mut cert_reader = BufReader::new(
        File::open(crt_path_obj).with_context(|| format!("open crtPath {crt_path}"))?,
    );
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .context("parse certificate PEM")?
        .into_iter()
        .collect();
    if certs.is_empty() {
        bail!("no certificates in {crt_path}");
    }

    // Prevent path traversal attacks by rejecting paths containing '..'.
    let key_path_obj = Path::new(key_path);
    if key_path_obj
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        bail!("Invalid input: {}", key_path_obj.display());
    }
    let mut key_reader = BufReader::new(
        File::open(key_path_obj).with_context(|| format!("open keyPath {key_path}"))?,
    );
    let key = rustls_pemfile::private_key(&mut key_reader)
        .context("parse private key PEM")?
        .ok_or_else(|| anyhow::anyhow!("no private key in {key_path}"))?;

    Ok((certs, key))
}
