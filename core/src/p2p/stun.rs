use anyhow::{anyhow, bail, Context, Result};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::{lookup_host, UdpSocket};
use tokio::time::timeout;

const STUN_BINDING_REQUEST: u16 = 0x0001;
const STUN_BINDING_SUCCESS_RESPONSE: u16 = 0x0101;
const STUN_MAGIC_COOKIE: u32 = 0x2112_A442;

const ATTR_MAPPED_ADDRESS: u16 = 0x0001;
const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;

/// Options for a STUN Binding query.
#[derive(Debug, Clone)]
pub struct StunQueryOptions {
    /// How long to wait for the server response.
    pub timeout: Duration,
}

impl Default for StunQueryOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(3),
        }
    }
}

/// Query a single STUN server and return the public `SocketAddr` seen by that server.
///
/// A new UDP socket is allocated for each call.  If you already hold a socket
/// that you intend to use for hole-punching, use [`query_public_addr_with_socket`]
/// instead so that the NAT mapping stays consistent.
pub async fn query_public_addr(server: &str, options: StunQueryOptions) -> Result<SocketAddr> {
    let server_addr = resolve_stun_server(server).await?;
    let bind_addr = match server_addr {
        SocketAddr::V4(_) => "0.0.0.0:0",
        SocketAddr::V6(_) => "[::]:0",
    };

    let socket = UdpSocket::bind(bind_addr)
        .await
        .with_context(|| format!("bind UDP socket for STUN server {server_addr}"))?;

    query_public_addr_with_socket(&socket, server_addr, options).await
}

/// Query multiple STUN servers concurrently and return all unique public addresses.
///
/// Failures are silently ignored (logged at `debug` level).  Returns an empty
/// `Vec` if every server fails.
pub async fn query_public_addrs(
    servers: &[String],
    options: StunQueryOptions,
) -> Vec<SocketAddr> {
    let mut out: Vec<SocketAddr> = Vec::new();

    for server in servers {
        match query_public_addr(server, options.clone()).await {
            Ok(addr) if !out.contains(&addr) => out.push(addr),
            Ok(_) => {}
            Err(e) => {
                tracing::debug!(server = %server, error = %e, "STUN query failed");
            }
        }
    }

    out
}

/// Run a STUN Binding Request over an existing UDP socket.
///
/// This is the preferred variant when hole-punching: using the same socket for
/// STUN and for the actual punch keeps the NAT binding stable.
pub async fn query_public_addr_with_socket(
    socket: &UdpSocket,
    server_addr: SocketAddr,
    options: StunQueryOptions,
) -> Result<SocketAddr> {
    let txid = new_transaction_id();
    let req = build_binding_request(txid);

    socket
        .send_to(&req, server_addr)
        .await
        .with_context(|| format!("send STUN Binding Request to {server_addr}"))?;

    let mut buf = [0u8; 1024];
    let (n, from) = timeout(options.timeout, socket.recv_from(&mut buf))
        .await
        .context("STUN receive timeout")?
        .with_context(|| format!("receive STUN response from {server_addr}"))?;

    if from.ip() != server_addr.ip() {
        tracing::debug!(
            expected = %server_addr,
            actual = %from,
            "received STUN response from unexpected source"
        );
    }

    parse_binding_response(&buf[..n], txid)
}

// ────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ────────────────────────────────────────────────────────────────────────────

async fn resolve_stun_server(server: &str) -> Result<SocketAddr> {
    let mut addrs = lookup_host(server)
        .await
        .with_context(|| format!("resolve STUN server {server}"))?;

    addrs
        .next()
        .ok_or_else(|| anyhow!("no resolved address for STUN server {server}"))
}

fn build_binding_request(txid: [u8; 12]) -> [u8; 20] {
    let mut buf = [0u8; 20];
    // Message type: Binding Request (0x0001)
    buf[0..2].copy_from_slice(&STUN_BINDING_REQUEST.to_be_bytes());
    // Message length: 0 (no attributes)
    buf[2..4].copy_from_slice(&0u16.to_be_bytes());
    // Magic cookie
    buf[4..8].copy_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes());
    // Transaction ID
    buf[8..20].copy_from_slice(&txid);
    buf
}

fn parse_binding_response(buf: &[u8], txid: [u8; 12]) -> Result<SocketAddr> {
    if buf.len() < 20 {
        bail!("STUN response too short ({} bytes)", buf.len());
    }

    let msg_type = u16::from_be_bytes([buf[0], buf[1]]);
    let msg_len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
    let magic_cookie = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

    if msg_type != STUN_BINDING_SUCCESS_RESPONSE {
        bail!("unexpected STUN message type: 0x{msg_type:04x}");
    }
    if magic_cookie != STUN_MAGIC_COOKIE {
        bail!("invalid STUN magic cookie: 0x{magic_cookie:08x}");
    }
    if buf[8..20] != txid {
        bail!("STUN transaction ID mismatch");
    }
    if buf.len() < 20 + msg_len {
        bail!(
            "truncated STUN message: need {} bytes, have {}",
            20 + msg_len,
            buf.len()
        );
    }

    let attrs = &buf[20..20 + msg_len];
    let mut offset = 0usize;
    let mut fallback_mapped: Option<SocketAddr> = None;

    while offset + 4 <= attrs.len() {
        let attr_type = u16::from_be_bytes([attrs[offset], attrs[offset + 1]]);
        let attr_len = u16::from_be_bytes([attrs[offset + 2], attrs[offset + 3]]) as usize;
        let value_start = offset + 4;
        let value_end = value_start + attr_len;

        if value_end > attrs.len() {
            bail!("truncated STUN attribute (type=0x{attr_type:04x})");
        }

        let value = &attrs[value_start..value_end];

        match attr_type {
            ATTR_XOR_MAPPED_ADDRESS => {
                // Preferred: return immediately on first XOR-MAPPED-ADDRESS
                return parse_xor_mapped_address(value, txid);
            }
            ATTR_MAPPED_ADDRESS => {
                // Keep as fallback in case XOR variant is absent
                fallback_mapped = Some(parse_mapped_address(value)?);
            }
            _ => {}
        }

        // Attributes are padded to 4-byte boundaries
        offset = align4(value_end);
    }

    fallback_mapped
        .ok_or_else(|| anyhow!("STUN response contains neither XOR-MAPPED-ADDRESS nor MAPPED-ADDRESS"))
}

fn parse_xor_mapped_address(value: &[u8], txid: [u8; 12]) -> Result<SocketAddr> {
    if value.len() < 4 {
        bail!("XOR-MAPPED-ADDRESS attribute too short");
    }

    let family = value[1];
    // Port is XOR-ed with the most-significant 16 bits of the magic cookie
    let x_port = u16::from_be_bytes([value[2], value[3]]);
    let port = x_port ^ ((STUN_MAGIC_COOKIE >> 16) as u16);

    match family {
        0x01 => {
            // IPv4: XOR with magic cookie
            if value.len() < 8 {
                bail!("XOR-MAPPED-ADDRESS IPv4 value too short");
            }
            let cookie = STUN_MAGIC_COOKIE.to_be_bytes();
            let ip = Ipv4Addr::new(
                value[4] ^ cookie[0],
                value[5] ^ cookie[1],
                value[6] ^ cookie[2],
                value[7] ^ cookie[3],
            );
            Ok(SocketAddr::new(IpAddr::V4(ip), port))
        }
        0x02 => {
            // IPv6: XOR with magic-cookie || transaction-id
            if value.len() < 20 {
                bail!("XOR-MAPPED-ADDRESS IPv6 value too short");
            }
            let mut raw = [0u8; 16];
            let cookie = STUN_MAGIC_COOKIE.to_be_bytes();
            for i in 0..4 {
                raw[i] = value[4 + i] ^ cookie[i];
            }
            for i in 0..12 {
                raw[4 + i] = value[8 + i] ^ txid[i];
            }
            Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::from(raw)), port))
        }
        other => bail!("unsupported address family in XOR-MAPPED-ADDRESS: 0x{other:02x}"),
    }
}

fn parse_mapped_address(value: &[u8]) -> Result<SocketAddr> {
    if value.len() < 4 {
        bail!("MAPPED-ADDRESS attribute too short");
    }

    let family = value[1];
    let port = u16::from_be_bytes([value[2], value[3]]);

    match family {
        0x01 => {
            if value.len() < 8 {
                bail!("MAPPED-ADDRESS IPv4 value too short");
            }
            let ip = Ipv4Addr::new(value[4], value[5], value[6], value[7]);
            Ok(SocketAddr::new(IpAddr::V4(ip), port))
        }
        0x02 => {
            if value.len() < 20 {
                bail!("MAPPED-ADDRESS IPv6 value too short");
            }
            let mut raw = [0u8; 16];
            raw.copy_from_slice(&value[4..20]);
            Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::from(raw)), port))
        }
        other => bail!("unsupported address family in MAPPED-ADDRESS: 0x{other:02x}"),
    }
}

/// Generate a 96-bit transaction ID derived from the current nanosecond timestamp.
/// Good enough for a minimal client that never retransmits concurrently.
fn new_transaction_id() -> [u8; 12] {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    // u128 is 16 bytes; take the lower 12 bytes (drop the top 4)
    let bytes = now.to_be_bytes();
    let mut txid = [0u8; 12];
    txid.copy_from_slice(&bytes[4..16]);
    txid
}

#[inline]
fn align4(n: usize) -> usize {
    (n + 3) & !3
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify XOR-MAPPED-ADDRESS decoding for a hand-crafted IPv4 example.
    ///
    /// x_port = 0x3326, cookie_hi = 0x2112  → port = 0x3326 ^ 0x2112 = 0x1234 = 4660
    /// x_ip   = [0x20, 0x10, 0xa7, 0x43], cookie = [0x21, 0x12, 0xa4, 0x42]
    ///        → ip  = [0x01, 0x02, 0x03, 0x01] = 1.2.3.1
    #[test]
    fn parse_xor_mapped_address_ipv4_ok() {
        let txid = [0x01u8; 12];
        let value: [u8; 8] = [
            0x00, 0x01, // reserved, family=IPv4
            0x33, 0x26, // x-port
            0x20, 0x10, 0xa7, 0x43, // x-ip
        ];
        let addr = parse_xor_mapped_address(&value, txid).unwrap();
        assert_eq!(addr, "1.2.3.1:4660".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn parse_binding_response_rejects_short_buf() {
        let result = parse_binding_response(&[0u8; 10], [0u8; 12]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_binding_response_rejects_wrong_type() {
        let mut buf = [0u8; 24];
        // msg_type = 0x0001 (request, not response)
        buf[0..2].copy_from_slice(&0x0001u16.to_be_bytes());
        buf[4..8].copy_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes());
        let result = parse_binding_response(&buf, [0u8; 12]);
        assert!(result.is_err());
    }
}
