//! P2P direct-tunnel helpers used by the orbien *client*.
//!
//! The server-side broker lives in `server/src/service/p2p_broker.rs`.
//! This module contains the client-side hole-punch logic that runs after
//! the server has exchanged addresses via `P2pReady`.

mod hole_punch;

pub use hole_punch::{HolePunchConfig, HolePunchResult, punch};
