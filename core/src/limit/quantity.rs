use anyhow::{bail, Result};

pub fn mbps_to_bytes_per_sec(mbps: f64) -> u64 {
    if mbps <= 0.0 || !mbps.is_finite() {
        return 0;
    }
    (mbps * 1_000_000.0 / 8.0).round() as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandwidthLimitSide {
    Client,
    Server,
}

impl BandwidthLimitSide {
    pub fn parse(s: &str) -> Self {
        if s.trim().eq_ignore_ascii_case("server") {
            Self::Server
        } else {
            Self::Client
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Server => "server",
        }
    }
}

pub fn parse_bandwidth_mbps(raw: &str) -> Result<f64> {
    let s = raw.trim();
    if s.is_empty() {
        return Ok(0.0);
    }
    let n: f64 = s
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid bandwidth (Mbps number expected), got {s:?}"))?;
    if !n.is_finite() || n < 0.0 {
        bail!("invalid bandwidth value: {s:?}");
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_zero_and_empty() {
        assert_eq!(parse_bandwidth_mbps("").unwrap(), 0.0);
        assert_eq!(parse_bandwidth_mbps("   ").unwrap(), 0.0);
        assert_eq!(parse_bandwidth_mbps("0").unwrap(), 0.0);
    }

    #[test]
    fn parse_valid_mbps() {
        assert_eq!(parse_bandwidth_mbps("1").unwrap(), 1.0);
        assert_eq!(parse_bandwidth_mbps("2.5").unwrap(), 2.5);
        assert_eq!(parse_bandwidth_mbps(" 512 ").unwrap(), 512.0);
    }

    #[test]
    fn parse_invalid() {
        assert!(parse_bandwidth_mbps("abc").is_err());
        assert!(parse_bandwidth_mbps("-1").is_err());
        assert!(parse_bandwidth_mbps("nan").is_err());
    }

    #[test]
    fn mbps_to_bytes() {
        assert_eq!(mbps_to_bytes_per_sec(8.0), 1_000_000);
        assert_eq!(mbps_to_bytes_per_sec(0.0), 0);
        assert_eq!(mbps_to_bytes_per_sec(-1.0), 0);
    }

    #[test]
    fn mode_parse() {
        assert_eq!(BandwidthLimitSide::parse("server"), BandwidthLimitSide::Server);
        assert_eq!(BandwidthLimitSide::parse("SERVER"), BandwidthLimitSide::Server);
        assert_eq!(BandwidthLimitSide::parse("Server"), BandwidthLimitSide::Server);
        assert_eq!(BandwidthLimitSide::parse("client"), BandwidthLimitSide::Client);
        assert_eq!(BandwidthLimitSide::parse("anything"), BandwidthLimitSide::Client);
        assert_eq!(BandwidthLimitSide::parse(""), BandwidthLimitSide::Client);
    }

    #[test]
    fn mode_as_str() {
        assert_eq!(BandwidthLimitSide::Server.as_str(), "server");
        assert_eq!(BandwidthLimitSide::Client.as_str(), "client");
    }
}
