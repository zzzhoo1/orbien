use anyhow::{bail, Result};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub const PROXY_PROTOCOL_MAX_HEADER: usize = 256;

const V2_SIG: [u8; 12] = [
    0x0d, 0x0a, 0x0d, 0x0a, 0x00, 0x0d, 0x0a, 0x51, 0x55, 0x49, 0x54, 0x0a,
];

pub fn parse_proxy_protocol_version(s: &str) -> Result<Option<&'static str>> {
    let s = s.trim();
    if s.is_empty() {
        Ok(None)
    } else if s.eq_ignore_ascii_case("v1") {
        Ok(Some("v1"))
    } else if s.eq_ignore_ascii_case("v2") {
        Ok(Some("v2"))
    } else {
        bail!("invalid proxyProtocolVersion {s:?}; use \"\" | \"v1\" | \"v2\"")
    }
}

pub fn build_proxy_protocol_header(
    src: SocketAddr,
    dst: SocketAddr,
    version: &str,
) -> Result<Vec<u8>> {
    match version {
        "v1" => Ok(build_v1(src, dst)?),
        "v2" => Ok(build_v2(src, dst)?),
        other => bail!("unsupported PROXY Protocol version: {other}"),
    }
}

fn build_v1(src: SocketAddr, dst: SocketAddr) -> Result<Vec<u8>> {
    let (family, src_ip, dst_ip) = match (src.ip(), dst.ip()) {
        (IpAddr::V4(a), IpAddr::V4(b)) => ("TCP4", a.to_string(), b.to_string()),
        (IpAddr::V6(a), IpAddr::V6(b)) => ("TCP6", a.to_string(), b.to_string()),
        (IpAddr::V4(a), IpAddr::V6(_)) => ("TCP4", a.to_string(), "127.0.0.1".into()),
        (IpAddr::V6(a), IpAddr::V4(_)) => ("TCP6", a.to_string(), "::1".into()),
    };
    Ok(format!(
        "PROXY {family} {src_ip} {dst_ip} {} {}\r\n",
        src.port(),
        dst.port()
    )
    .into_bytes())
}

fn build_v2(src: SocketAddr, dst: SocketAddr) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(16 + 36);
    out.extend_from_slice(&V2_SIG);
    out.push(0x21);
    match (src.ip(), dst.ip()) {
        (IpAddr::V4(s), IpAddr::V4(d)) => {
            out.push(0x11);
            out.extend_from_slice(&12u16.to_be_bytes());
            out.extend_from_slice(&s.octets());
            out.extend_from_slice(&d.octets());
            out.extend_from_slice(&src.port().to_be_bytes());
            out.extend_from_slice(&dst.port().to_be_bytes());
        }
        (IpAddr::V6(s), IpAddr::V6(d)) => {
            out.push(0x21);
            out.extend_from_slice(&36u16.to_be_bytes());
            out.extend_from_slice(&s.octets());
            out.extend_from_slice(&d.octets());
            out.extend_from_slice(&src.port().to_be_bytes());
            out.extend_from_slice(&dst.port().to_be_bytes());
        }
        (IpAddr::V4(s), _) => {
            let d = Ipv4Addr::LOCALHOST;
            out.push(0x11);
            out.extend_from_slice(&12u16.to_be_bytes());
            out.extend_from_slice(&s.octets());
            out.extend_from_slice(&d.octets());
            out.extend_from_slice(&src.port().to_be_bytes());
            out.extend_from_slice(&dst.port().to_be_bytes());
        }
        (IpAddr::V6(s), _) => {
            let d = Ipv6Addr::LOCALHOST;
            out.push(0x21);
            out.extend_from_slice(&36u16.to_be_bytes());
            out.extend_from_slice(&s.octets());
            out.extend_from_slice(&d.octets());
            out.extend_from_slice(&src.port().to_be_bytes());
            out.extend_from_slice(&dst.port().to_be_bytes());
        }
    }
    Ok(out)
}

#[derive(Debug, Clone)]
pub struct ParsedProxyHeader {
    pub src: SocketAddr,
    pub dst: SocketAddr,
    pub header_len: usize,
}

#[derive(Debug)]
pub enum PpConsume {
    NotProxy,

    Incomplete,

    Done(ParsedProxyHeader),
}

pub fn try_consume_proxy_protocol(buf: &[u8]) -> Result<PpConsume> {
    if buf.is_empty() {
        return Ok(PpConsume::Incomplete);
    }

    if buf[0] == b'P' {
        if buf.len() < 6 {
            return Ok(if b"PROXY ".starts_with(buf) {
                PpConsume::Incomplete
            } else {
                PpConsume::NotProxy
            });
        }
        if !buf.starts_with(b"PROXY ") {
            return Ok(PpConsume::NotProxy);
        }
        return parse_v1(buf);
    }

    if buf[0] == 0x0d {
        if buf.len() < 12 {
            return Ok(if V2_SIG.starts_with(buf) {
                PpConsume::Incomplete
            } else {
                PpConsume::NotProxy
            });
        }
        if !buf.starts_with(&V2_SIG) {
            return Ok(PpConsume::NotProxy);
        }
        return parse_v2(buf);
    }
    Ok(PpConsume::NotProxy)
}

fn parse_v1(buf: &[u8]) -> Result<PpConsume> {
    let Some(end) = buf.windows(2).position(|w| w == b"\r\n") else {
        return Ok(if buf.len() >= 108 {
            bail!("invalid PROXY v1 header (no CRLF)");
        } else {
            PpConsume::Incomplete
        });
    };
    let line =
        std::str::from_utf8(&buf[..end]).map_err(|_| anyhow::anyhow!("PROXY v1 not utf8"))?;
    let header_len = end + 2;
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() >= 2 && parts[1] == "UNKNOWN" {
        return Ok(PpConsume::Done(ParsedProxyHeader {
            src: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            dst: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            header_len,
        }));
    }
    if parts.len() < 6 {
        bail!("invalid PROXY v1 line: {line}");
    }
    let src_ip: IpAddr = parts[2].parse()?;
    let dst_ip: IpAddr = parts[3].parse()?;
    let src_port: u16 = parts[4].parse()?;
    let dst_port: u16 = parts[5].parse()?;
    Ok(PpConsume::Done(ParsedProxyHeader {
        src: SocketAddr::new(src_ip, src_port),
        dst: SocketAddr::new(dst_ip, dst_port),
        header_len,
    }))
}

fn parse_v2(buf: &[u8]) -> Result<PpConsume> {
    if buf.len() < 16 {
        return Ok(PpConsume::Incomplete);
    }
    let ver_cmd = buf[12];
    let version = ver_cmd >> 4;
    let cmd = ver_cmd & 0x0f;
    if version != 0x02 {
        bail!("unsupported PROXY v2 version {version}");
    }
    let fam_prot = buf[13];
    let addr_len = u16::from_be_bytes([buf[14], buf[15]]) as usize;
    let total = 16 + addr_len;
    if buf.len() < total {
        return Ok(PpConsume::Incomplete);
    }
    if cmd == 0x00 {
        return Ok(PpConsume::Done(ParsedProxyHeader {
            src: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            dst: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            header_len: total,
        }));
    }
    if cmd != 0x01 {
        bail!("unsupported PROXY v2 command {cmd}");
    }
    let addr = &buf[16..total];
    let (src, dst) = match fam_prot {
        0x11 => {
            if addr.len() < 12 {
                bail!("PROXY v2 IPv4 address block short");
            }
            let s = Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]);
            let d = Ipv4Addr::new(addr[4], addr[5], addr[6], addr[7]);
            let sp = u16::from_be_bytes([addr[8], addr[9]]);
            let dp = u16::from_be_bytes([addr[10], addr[11]]);
            (
                SocketAddr::new(IpAddr::V4(s), sp),
                SocketAddr::new(IpAddr::V4(d), dp),
            )
        }
        0x21 => {
            if addr.len() < 36 {
                bail!("PROXY v2 IPv6 address block short");
            }
            let mut so = [0u8; 16];
            let mut do_ = [0u8; 16];
            so.copy_from_slice(&addr[0..16]);
            do_.copy_from_slice(&addr[16..32]);
            let sp = u16::from_be_bytes([addr[32], addr[33]]);
            let dp = u16::from_be_bytes([addr[34], addr[35]]);
            (
                SocketAddr::new(IpAddr::V6(Ipv6Addr::from(so)), sp),
                SocketAddr::new(IpAddr::V6(Ipv6Addr::from(do_)), dp),
            )
        }
        other => bail!("unsupported PROXY v2 family/protocol {other:#x}"),
    };
    Ok(PpConsume::Done(ParsedProxyHeader {
        src,
        dst,
        header_len: total,
    }))
}

pub fn addrs_from_start_data_conn(
    src_ip: &str,
    src_port: u16,
    dst_ip: &str,
    dst_port: u16,
    fallback_dst_port: u16,
) -> Option<(SocketAddr, SocketAddr)> {
    if src_ip.is_empty() {
        return None;
    }
    let src_ip: IpAddr = src_ip.parse().ok()?;
    let src = SocketAddr::new(src_ip, src_port);
    let dst_ip: IpAddr = if dst_ip.is_empty() {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        dst_ip.parse().ok()?
    };
    let dst_port = if dst_port == 0 {
        fallback_dst_port
    } else {
        dst_port
    };
    Some((src, SocketAddr::new(dst_ip, dst_port)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn parse_version_valid() {
        assert_eq!(parse_proxy_protocol_version("").unwrap(), None);
        assert_eq!(parse_proxy_protocol_version("v1").unwrap(), Some("v1"));
        assert_eq!(parse_proxy_protocol_version("V2").unwrap(), Some("v2"));
        assert!(parse_proxy_protocol_version("v3").is_err());
    }

    #[test]
    fn build_v1_tcp4() {
        let src: SocketAddr = "1.2.3.4:1234".parse().unwrap();
        let dst: SocketAddr = "5.6.7.8:80".parse().unwrap();
        let hdr = build_proxy_protocol_header(src, dst, "v1").unwrap();
        assert_eq!(
            String::from_utf8(hdr).unwrap(),
            "PROXY TCP4 1.2.3.4 5.6.7.8 1234 80\r\n"
        );
    }

    #[test]
    fn build_v1_tcp6() {
        let src: SocketAddr = "[::1]:443".parse().unwrap();
        let dst: SocketAddr = "[::2]:80".parse().unwrap();
        let hdr = build_proxy_protocol_header(src, dst, "v1").unwrap();
        let s = String::from_utf8(hdr).unwrap();
        assert!(s.starts_with("PROXY TCP6 ::1 ::2 443 80\r\n"));
    }

    #[test]
    fn build_v1_mixed_family() {
        let src: SocketAddr = "1.2.3.4:1234".parse().unwrap();
        let dst: SocketAddr = "[::1]:80".parse().unwrap();
        let hdr = build_proxy_protocol_header(src, dst, "v1").unwrap();
        let s = String::from_utf8(hdr).unwrap();
        assert_eq!(s, "PROXY TCP4 1.2.3.4 127.0.0.1 1234 80\r\n");
    }

    #[test]
    fn build_v2_tcp4() {
        let src: SocketAddr = "1.2.3.4:1234".parse().unwrap();
        let dst: SocketAddr = "5.6.7.8:80".parse().unwrap();
        let hdr = build_proxy_protocol_header(src, dst, "v2").unwrap();
        // 16-byte header + 12-byte IPv4 block
        assert_eq!(hdr.len(), 28);
        assert_eq!(&hdr[0..12], &V2_SIG);
        assert_eq!(hdr[12], 0x21); // version 2, PROXY
        assert_eq!(hdr[13], 0x11); // AF_INET / STREAM
        assert_eq!(u16::from_be_bytes([hdr[14], hdr[15]]), 12);
        // Parse it back
        match try_consume_proxy_protocol(&hdr).unwrap() {
            PpConsume::Done(p) => {
                assert_eq!(p.src, src);
                assert_eq!(p.dst, dst);
                assert_eq!(p.header_len, 28);
            }
            other => panic!("expected Done, got {other:?}"),
        }
    }

    #[test]
    fn build_v2_tcp6() {
        let src: SocketAddr = "[2001:db8::1]:443".parse().unwrap();
        let dst: SocketAddr = "[2001:db8::2]:80".parse().unwrap();
        let hdr = build_proxy_protocol_header(src, dst, "v2").unwrap();
        assert_eq!(hdr.len(), 16 + 36);
        assert_eq!(hdr[13], 0x21); // AF_INET6 / STREAM
        match try_consume_proxy_protocol(&hdr).unwrap() {
            PpConsume::Done(p) => {
                assert_eq!(p.src, src);
                assert_eq!(p.dst, dst);
            }
            other => panic!("expected Done, got {other:?}"),
        }
    }

    #[test]
    fn consume_not_proxy() {
        assert!(matches!(
            try_consume_proxy_protocol(b"GET / HTTP/1.1\r\n").unwrap(),
            PpConsume::NotProxy
        ));
        assert!(matches!(
            try_consume_proxy_protocol(b"").unwrap(),
            PpConsume::Incomplete
        ));
    }

    #[test]
    fn consume_v1_unknown() {
        let hdr = b"PROXY UNKNOWN\r\n";
        match try_consume_proxy_protocol(hdr).unwrap() {
            PpConsume::Done(p) => {
                assert_eq!(p.src, SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0));
                assert_eq!(p.header_len, hdr.len());
            }
            other => panic!("expected Done, got {other:?}"),
        }
    }

    #[test]
    fn consume_incomplete_v1() {
        assert!(matches!(
            try_consume_proxy_protocol(b"PROXY TCP4 1.2.3.4").unwrap(),
            PpConsume::Incomplete
        ));
    }

    #[test]
    fn addrs_from_start_data_conn_basic() {
        let r = addrs_from_start_data_conn("1.2.3.4", 1234, "5.6.7.8", 80, 8080).unwrap();
        assert_eq!(r.0, "1.2.3.4:1234".parse::<SocketAddr>().unwrap());
        assert_eq!(r.1, "5.6.7.8:80".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn addrs_from_start_data_conn_empty_src() {
        assert!(addrs_from_start_data_conn("", 0, "", 0, 8080).is_none());
    }

    #[test]
    fn addrs_from_start_data_conn_fallback() {
        let r = addrs_from_start_data_conn("1.2.3.4", 1234, "", 0, 8080).unwrap();
        assert_eq!(r.0, "1.2.3.4:1234".parse::<SocketAddr>().unwrap());
        assert_eq!(r.1, "127.0.0.1:8080".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn addrs_from_start_data_conn_invalid() {
        assert!(addrs_from_start_data_conn("not-an-ip", 1, "", 0, 8080).is_none());
    }
}
