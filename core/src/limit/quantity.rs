use anyhow::{bail, Result};

pub const KB: u64 = 1024;
pub const MB: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandwidthLimitMode {
    Client,
    Server,
}

impl BandwidthLimitMode {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "server" => Self::Server,
            _ => Self::Client,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Server => "server",
        }
    }
}

pub fn parse_bandwidth_limit(s: &str) -> Result<u64> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(0);
    }
    if let Some(num) = s.strip_suffix("MB").or_else(|| s.strip_suffix("mb")) {
        let f: f64 = num.trim().parse()?;
        return Ok((f * MB as f64) as u64);
    }
    if let Some(num) = s.strip_suffix("KB").or_else(|| s.strip_suffix("kb")) {
        let f: f64 = num.trim().parse()?;
        return Ok((f * KB as f64) as u64);
    }
    bail!("bandwidthLimit unit not supported (use KB or MB), got {s:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_zero_and_empty() {
        assert_eq!(parse_bandwidth_limit("").unwrap(), 0);
        assert_eq!(parse_bandwidth_limit("   ").unwrap(), 0);
        // Bare number without unit is rejected.
        assert!(parse_bandwidth_limit("0").is_err());
    }

    #[test]
    fn parse_kb() {
        assert_eq!(parse_bandwidth_limit("1KB").unwrap(), KB);
        assert_eq!(parse_bandwidth_limit("2kb").unwrap(), 2 * KB);
        assert_eq!(parse_bandwidth_limit(" 512 KB ").unwrap(), 512 * KB);
    }

    #[test]
    fn parse_mb() {
        assert_eq!(parse_bandwidth_limit("1MB").unwrap(), MB);
        assert_eq!(parse_bandwidth_limit("1.5mb").unwrap(), (1.5 * MB as f64) as u64);
        assert_eq!(parse_bandwidth_limit("0.5MB").unwrap(), (0.5 * MB as f64) as u64);
    }

    #[test]
    fn parse_unsupported_unit() {
        assert!(parse_bandwidth_limit("1GB").is_err());
        assert!(parse_bandwidth_limit("10").is_err());
        assert!(parse_bandwidth_limit("abc").is_err());
    }

    #[test]
    fn mode_parse() {
        assert_eq!(BandwidthLimitMode::parse("server"), BandwidthLimitMode::Server);
        assert_eq!(BandwidthLimitMode::parse("SERVER"), BandwidthLimitMode::Server);
        assert_eq!(BandwidthLimitMode::parse("Server"), BandwidthLimitMode::Server);
        assert_eq!(BandwidthLimitMode::parse("client"), BandwidthLimitMode::Client);
        assert_eq!(BandwidthLimitMode::parse("anything"), BandwidthLimitMode::Client);
        assert_eq!(BandwidthLimitMode::parse(""), BandwidthLimitMode::Client);
    }

    #[test]
    fn mode_as_str() {
        assert_eq!(BandwidthLimitMode::Server.as_str(), "server");
        assert_eq!(BandwidthLimitMode::Client.as_str(), "client");
    }
}
