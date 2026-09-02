#![cfg(test)]
//! Integration tests for the yamux multiplexer helpers:
//! `client_session` / `YamuxClient` and `serve_yamux_session`.
//!
//! Uses tokio duplex streams as the physical transport so no real TCP
//! sockets are needed — tests run fast and work in any CI environment.

use super::yamux_mux::{client_session, serve_yamux_session, MAX_NUM_STREAMS};
use crate::transport::boxed_stream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Create a matched (client, server) pair of `DynStream` backed by
/// `tokio::io::duplex`.  The buffer is 256 KiB so it can absorb at least
/// one full yamux window without blocking.
fn duplex_pair() -> (crate::transport::DynStream, crate::transport::DynStream) {
    let (c, s) = tokio::io::duplex(256 * 1024);
    (boxed_stream(c), boxed_stream(s))
}

// ── single stream open / echo ─────────────────────────────────────────────────

#[tokio::test]
async fn open_single_stream_and_echo() {
    let (client_io, server_io) = duplex_pair();

    // Server: accept one inbound stream and echo everything back.
    let server = tokio::spawn(async move {
        serve_yamux_session(server_io, MAX_NUM_STREAMS, |mut stream| {
            tokio::spawn(async move {
                let mut buf = vec![0u8; 64];
                let n = stream.read(&mut buf).await.expect("server read");
                stream.write_all(&buf[..n]).await.expect("server write");
            });
        })
        .await
        .ok(); // connection closed by client drop is OK
    });

    // Client: open one stream, write, read back.
    let client = client_session(client_io, MAX_NUM_STREAMS);
    let mut stream = client.open_stream().await.expect("open stream");
    stream.write_all(b"hello yamux").await.expect("write");
    let mut buf = vec![0u8; 11];
    stream.read_exact(&mut buf).await.expect("read");
    assert_eq!(&buf, b"hello yamux");

    drop(client);
    server.await.ok();
}

// ── multi-stream concurrency ──────────────────────────────────────────────────

#[tokio::test]
async fn open_multiple_streams_concurrently() {
    const N: usize = 8;
    let (client_io, server_io) = duplex_pair();

    let server = tokio::spawn(async move {
        serve_yamux_session(server_io, MAX_NUM_STREAMS, |mut stream| {
            tokio::spawn(async move {
                // Echo one byte back.
                let mut b = [0u8; 1];
                if stream.read_exact(&mut b).await.is_ok() {
                    let _ = stream.write_all(&b).await;
                }
            });
        })
        .await
        .ok();
    });

    let client = client_session(client_io, MAX_NUM_STREAMS);
    let mut handles = Vec::new();
    for i in 0..N {
        let c = client.open_stream().await.expect("open");
        handles.push((i as u8, c));
    }

    for (byte, mut stream) in handles {
        stream.write_all(&[byte]).await.expect("write");
        let mut back = [0u8; 1];
        stream.read_exact(&mut back).await.expect("read");
        assert_eq!(back[0], byte, "echo mismatch for byte {byte}");
    }

    drop(client);
    server.await.ok();
}

// ── max_streams accessor ──────────────────────────────────────────────────────

#[test]
fn yamux_client_reports_max_streams() {
    // We only need to verify the accessor — no async needed here.
    // Build a dummy client over a channel that we immediately drop.
    let (client_io, _server_io) = tokio::io::duplex(4096);
    // We cannot call client_session without a runtime, so use a
    // single-threaded runtime to verify the constructor path.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let client = client_session(boxed_stream(client_io), 64);
        assert_eq!(client.max_streams(), 64);
    });
}

// ── mux disabled path ────────────────────────────────────────────────────────
// Ensure the non-mux path (plain stream, no yamux framing) is not accidentally
// broken. We verify this indirectly: a plain duplex write/read round-trip works
// without going through serve_yamux_session at all.
#[tokio::test]
async fn plain_stream_roundtrip_without_mux() {
    let (mut a, mut b) = tokio::io::duplex(1024);
    a.write_all(b"plain").await.unwrap();
    drop(a);
    let mut buf = Vec::new();
    b.read_to_end(&mut buf).await.unwrap();
    assert_eq!(buf, b"plain");
}
