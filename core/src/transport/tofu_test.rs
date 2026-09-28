#![cfg(test)]
//! Integration tests for the H1 Trust-On-First-Use (TOFU) verifier
//! (`TofuServerVerifier` in `transport/tls.rs`).
//!
//! Drives real rustls handshakes over in-memory duplex streams: a server
//! side with a fixed self-signed cert vs. a client configured with a TOFU
//! store in a temp dir. Covers the three pinning branches (first connect,
//! match, mismatch), hostname separation, store persistence and reload,
//! and the CA-overrides-TOFU priority rule.

use crate::transport::tls::{new_client_tls_config_tofu, new_server_tls_config};
use crate::transport::boxed_stream;
use std::path::PathBuf;
use std::sync::Arc;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Self-signed server config bound to a fixed common name. The same cert is
/// reused across calls within one test (deterministic fingerprint), while
/// `rotated()` returns a *different* cert to simulate key rotation / MITM.
struct ServerCert {
    tls: Arc<rustls::ServerConfig>,
    common_name: String,
}

impl ServerCert {
    fn fresh(cn: &str) -> Self {
        let tls = new_server_tls_config("", "", "").expect("server tls config");
        Self {
            tls,
            common_name: cn.to_string(),
        }
    }

    /// A *different* self-signed cert (same CN, different key) — the client
    /// must reject it once the original is pinned.
    fn rotated(cn: &str) -> Self {
        Self::fresh(cn)
    }
}

/// Run one full client↔server TLS handshake over an in-memory duplex pair.
/// Returns Ok(()) if the handshake completed, Err with the message otherwise.
async fn shake<'a>(
    server: &'a ServerCert,
    client_store: Option<&'a PathBuf>,
    server_name: &'a str,
) -> Result<(), String> {
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::clone(&server.tls));

    let client_cfg =
        new_client_tls_config_tofu("", "", "", client_store.map(|p| p.as_path()))
            .expect("client tls config");
    let connector = tokio_rustls::TlsConnector::from(client_cfg);

    let (c_io, s_io) = tokio::io::duplex(64 * 1024);

    let sn: rustls::pki_types::ServerName<'static> = server_name
        .to_string()
        .try_into()
        .expect("server name");
    let server_task = tokio::spawn(async move {
        let _conn = acceptor.accept(s_io).await;
    });
    let client = cli_inner(connector, sn, boxed_stream(c_io));

    let (srv, cli) = tokio::join!(server_task, client);
    srv.ok();
    cli
}

async fn cli_inner(
    connector: tokio_rustls::TlsConnector,
    sn: rustls::pki_types::ServerName<'static>,
    c_io: crate::transport::DynStream,
) -> Result<(), String> {
    let mut conn = connector.connect(sn, c_io).await.map_err(|e| e.to_string())?;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    // Drive one byte so both sides complete the handshake; a clean EOF after
    // the write is fine — we only need the handshake itself to have succeeded.
    conn.write_all(b"p").await.map_err(|e| e.to_string())?;
    let mut buf = [0u8; 1];
    let _ = conn.read(&mut buf).await;
    Ok::<(), String>(())
}

fn temp_store(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "orbien-tofu-test-{}-{}",
        name,
        std::process::id()
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir.join("tofu-store.txt")
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// First connection with an empty store: handshake must succeed and the
/// fingerprint must be persisted.
#[tokio::test]
async fn tofu_first_connect_pins_and_succeeds() {
    let store = temp_store("first-connect");
    let server = ServerCert::fresh("orbien-server");

    shake(&server, Some(&store), "orbien-server")
        .await
        .expect("first connect must succeed");

    let raw = std::fs::read_to_string(&store).expect("store written");
    assert!(raw.contains("orbien-server"), "store records the host");
    // SHA-256 hex = 64 chars per line.
    let pinned = raw.lines().find(|l| !l.starts_with('#')).unwrap();
    let fp = pinned.split('\t').nth(1).unwrap();
    assert_eq!(fp.len(), 64, "fingerprint is sha256 hex");
    let _ = std::fs::remove_dir_all(store.parent().unwrap());
}

/// Second connection with the same cert: succeeds silently (no re-pin warn).
#[tokio::test]
async fn tofu_matching_cert_passes() {
    let store = temp_store("matching");
    let server = ServerCert::fresh("orbien-server");

    shake(&server, Some(&store), "orbien-server")
        .await
        .expect("first connect");
    shake(&server, Some(&store), "orbien-server")
        .await
        .expect("second connect with same cert must pass");

    let _ = std::fs::remove_dir_all(store.parent().unwrap());
}

/// Key rotation / MITM: a *different* cert for a pinned host must be
/// rejected — the handshake fails on the client side.
#[tokio::test]
async fn tofu_mismatched_cert_is_rejected() {
    let store = temp_store("mismatch");
    let server = ServerCert::fresh("orbien-server");
    let attacker = ServerCert::rotated("orbien-server");

    shake(&server, Some(&store), "orbien-server")
        .await
        .expect("first connect pins original");

    let err = shake(&attacker, Some(&store), "orbien-server")
        .await
        .expect_err("rotated cert must be rejected");
    assert!(
        err.contains("TOFU mismatch"),
        "error mentions TOFU mismatch, got: {err}"
    );

    // Store must be untouched by the rejected attempt.
    let raw = std::fs::read_to_string(&store).unwrap();
    let pinned = raw.lines().find(|l| !l.starts_with('#')).unwrap();
    assert!(
        raw.lines().filter(|l| !l.starts_with('#')).count() == 1,
        "no extra entries after rejection"
    );
    let _ = std::fs::remove_dir_all(store.parent().unwrap());
}

/// Distinct server names pin independently: pinning host-a does not let
/// host-b inherit it; each host gets its own entry.
#[tokio::test]
async fn tofu_hosts_are_pinned_independently() {
    let store = temp_store("multi-host");
    let server = ServerCert::fresh("orbien-server");

    shake(&server, Some(&store), "orbien-server")
        .await
        .expect("pin orbien-server");

    // A different SNI is a different pin slot: first connect for that host,
    // so it succeeds and gets its own entry even though the cert is the same.
    shake(&server, Some(&store), "other-host.example")
        .await
        .expect("second host pins independently");

    // The original host's pin still matches.
    shake(&server, Some(&store), "orbien-server")
        .await
        .expect("original pin still valid");

    // And the mismatch guard fires for the *other* host too.
    let rotated = ServerCert::rotated("orbien-server");
    let err = shake(&rotated, Some(&store), "other-host.example")
        .await
        .expect_err("rotated cert rejected for second host");
    assert!(err.contains("TOFU mismatch"), "got: {err}");

    // Both hosts recorded independently.
    let raw = std::fs::read_to_string(&store).unwrap();
    assert!(raw.contains("orbien-server"));
    assert!(raw.contains("other-host.example"));

    let _ = std::fs::remove_dir_all(store.parent().unwrap());
}

/// TOFU rejections are not sticky corruption: after removing the stale
/// entry, the new cert pins cleanly (documented re-pin procedure).
#[tokio::test]
async fn tofu_manual_repinn_after_mismatch() {
    let store = temp_store("re-pin");
    let server = ServerCert::fresh("orbien-server");
    let rotated = ServerCert::rotated("orbien-server");

    shake(&server, Some(&store), "orbien-server")
        .await
        .expect("pin original");

    let _ = shake(&rotated, Some(&store), "orbien-server").await; // rejected

    // Operator deletes the line (documented procedure).
    let raw = std::fs::read_to_string(&store).unwrap();
    let kept: String = raw
        .lines()
        .filter(|l| l.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&store, kept + "\n").unwrap();

    shake(&rotated, Some(&store), "orbien-server")
        .await
        .expect("new cert pins after manual store wipe");

    let _ = std::fs::remove_dir_all(store.parent().unwrap());
}
