mod quantity;
mod stream;
mod token_bucket;

use std::sync::Arc;

pub use quantity::{mbps_to_bytes_per_sec, parse_bandwidth_mbps, BandwidthLimitSide};
pub use stream::{maybe_limit, LimitedStream};
pub use token_bucket::BandwidthLimiter;

pub fn limiter_if_side(
    bandwidth_mbps: f64,
    bandwidth_limit_side: &str,
    want: BandwidthLimitSide,
) -> anyhow::Result<Option<Arc<BandwidthLimiter>>> {
    let bytes = mbps_to_bytes_per_sec(bandwidth_mbps);
    if bytes == 0 {
        return Ok(None);
    }
    if BandwidthLimitSide::parse(bandwidth_limit_side) != want {
        return Ok(None);
    }
    Ok(Some(Arc::new(BandwidthLimiter::new(bytes))))
}
