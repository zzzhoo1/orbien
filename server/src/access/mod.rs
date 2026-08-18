use anyhow::{bail, Result};
use orbien_core::net::{try_consume_proxy_protocol, PpConsume, PROXY_PROTOCOL_MAX_HEADER};
use orbien_core::tls::PrefixedStream;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct AccessPolicy {
    pub proxy_protocol: bool,
    pub trusted_proxy_cidrs: Vec<Cidr>,
    pub deny_src_cidrs: Vec<Cidr>,
    pub pp_header_timeout: Duration,
}

impl AccessPolicy {
    pub fn from_server_config(cfg: &orbien_core::config::ServerConfig) -> Result<Self> {
        let trusted = cfg
            .proxy_protocol_trusted_cidrs
            .iter()
            .map(|s| Cidr::parse(s))
            .collect::<Result<Vec<_>>>()?;
        let deny = cfg
            .deny_src_cidrs
            .iter()
            .map(|s| Cidr::parse(s))
            .collect::<Result<Vec<_>>>()?;
        if cfg.proxy_protocol && trusted.is_empty() {
            tracing::warn!(
                "proxyProtocol=true but proxyProtocolTrustedCidrs empty — \
                 PROXY protocol headers are ignored until trusted CIDRs are set"
            );
        }
        Ok(Self {
            proxy_protocol: cfg.proxy_protocol,
            trusted_proxy_cidrs: trusted,
            deny_src_cidrs: deny,
            pp_header_timeout: Duration::from_secs(cfg.proxy_protocol_timeout_secs.max(1)),
        })
    }

    pub fn is_trusted_proxy(&self, ip: IpAddr) -> bool {
        !self.trusted_proxy_cidrs.is_empty()
            && self.trusted_proxy_cidrs.iter().any(|c| c.contains(ip))
    }

    pub fn is_denied(&self, ip: IpAddr) -> bool {
        !self.deny_src_cidrs.is_empty() && self.deny_src_cidrs.iter().any(|c| c.contains(ip))
    }
}

pub struct VisitorConn {
    pub stream: PrefixedStream<TcpStream>,
    pub peer: SocketAddr,
    pub visitor: SocketAddr,
    pub local: Option<SocketAddr>,
}

pub async fn prepare_visitor(
    stream: TcpStream,
    peer: SocketAddr,
    policy: &AccessPolicy,
) -> Result<VisitorConn> {
    let local = stream.local_addr().ok();
    let (stream, mut visitor) = if policy.proxy_protocol && policy.is_trusted_proxy(peer.ip()) {
        read_optional_pp(stream, peer, policy.pp_header_timeout).await?
    } else {
        (PrefixedStream::new(Vec::new(), stream), peer)
    };

    if visitor.ip().is_unspecified() {
        visitor = peer;
    }

    if policy.is_denied(visitor.ip()) {
        tracing::info!(visitor = %visitor.ip(), peer = %peer, "denied by denySrcCidrs");
        bail!("visitor {} denied by denySrcCidrs", visitor.ip());
    }

    Ok(VisitorConn {
        stream,
        peer,
        visitor,
        local,
    })
}

async fn read_optional_pp(
    mut stream: TcpStream,
    peer: SocketAddr,
    hdr_timeout: Duration,
) -> Result<(PrefixedStream<TcpStream>, SocketAddr)> {
    let mut buf = vec![0u8; PROXY_PROTOCOL_MAX_HEADER];
    let mut filled = 0usize;

    loop {
        match try_consume_proxy_protocol(&buf[..filled])? {
            PpConsume::Done(parsed) => {
                let leftover = buf[parsed.header_len..filled].to_vec();
                tracing::debug!(
                    peer = %peer,
                    visitor = %parsed.src,
                    header_len = parsed.header_len,
                    "PROXY protocol accepted from trusted peer"
                );
                return Ok((PrefixedStream::new(leftover, stream), parsed.src));
            }
            PpConsume::NotProxy => {
                let leftover = buf[..filled].to_vec();
                return Ok((PrefixedStream::new(leftover, stream), peer));
            }
            PpConsume::Incomplete => {
                if filled >= buf.len() {
                    bail!("PROXY protocol header incomplete / too large");
                }
                let n = timeout(hdr_timeout, stream.read(&mut buf[filled..]))
                    .await
                    .map_err(|_| anyhow::anyhow!("timeout waiting for PROXY protocol header"))??;
                if n == 0 {
                    bail!("connection closed while reading PROXY protocol");
                }
                filled += n;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cidr {
    addr: IpAddr,
    prefix: u8,
}

impl Cidr {
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        if let Some((ip, pref)) = s.split_once('/') {
            let addr: IpAddr = ip.parse()?;
            let prefix: u8 = pref.parse()?;
            let max = match addr {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            };
            if prefix > max {
                bail!("CIDR prefix {prefix} too large for {addr}");
            }
            Ok(Self { addr, prefix })
        } else {
            let addr: IpAddr = s.parse()?;
            let prefix = match addr {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            };
            Ok(Self { addr, prefix })
        }
    }

    pub fn contains(&self, ip: IpAddr) -> bool {
        match (self.addr, ip) {
            (IpAddr::V4(net), IpAddr::V4(ip)) => ipv4_in_cidr(net, self.prefix, ip),
            (IpAddr::V6(net), IpAddr::V6(ip)) => ipv6_in_cidr(net, self.prefix, ip),
            _ => false,
        }
    }
}

fn ipv4_in_cidr(net: Ipv4Addr, prefix: u8, ip: Ipv4Addr) -> bool {
    if prefix == 0 {
        return true;
    }
    let mask = if prefix >= 32 {
        u32::MAX
    } else {
        !((1u32 << (32 - prefix)) - 1)
    };
    (u32::from(net) & mask) == (u32::from(ip) & mask)
}

fn ipv6_in_cidr(net: Ipv6Addr, prefix: u8, ip: Ipv6Addr) -> bool {
    if prefix == 0 {
        return true;
    }
    let net_o = net.octets();
    let ip_o = ip.octets();
    let full = (prefix / 8) as usize;
    let rem = prefix % 8;
    if net_o[..full] != ip_o[..full] {
        return false;
    }
    if rem == 0 {
        return true;
    }
    let mask = 0xffu8 << (8 - rem);
    (net_o[full] & mask) == (ip_o[full] & mask)
}
