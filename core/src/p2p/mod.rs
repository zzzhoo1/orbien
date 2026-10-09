//! P2P direct-tunnel helpers used by the orbien *client*.
//!
//! The server-side broker lives in `server/src/service/p2p_broker.rs`.
//! This module contains the client-side hole-punch logic that runs after
//! the server has exchanged addresses via `P2pReady`.

mod hole_punch;
pub mod stun;

pub use hole_punch::{parse_candidates, punch, HolePunchConfig, HolePunchResult};
pub use stun::{query_public_addr, query_public_addrs, query_public_addr_with_socket, StunQueryOptions};
