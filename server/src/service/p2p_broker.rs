//! Server-side P2P broker — coordinates the address-exchange handshake
//! between two orbien clients that want to establish a direct tunnel.
//!
//! # Protocol flow
//!
//! ```text
//! Client A (initiator)          Broker                  Client B (responder)
//!   │                              │                              │
//!   │── P2pReq {peer=B, token} ───>│                              │
//!   │                              │── P2pInfo {token, peer_addr=A_obs} ──>│
//!   │<─ P2pInfo {token, peer_addr=B_obs} ──────────────────────────│
//!   │── P2pAddr {token, candidates_A} ───────────────────────────>│  (via broker)
//!   │                              │<─ P2pAddr {token, candidates_B} ────────│
//!   │<─ P2pReady ──────────────────│── P2pReady ─────────────────>│
//!   │                              │                              │
//!   │  (UDP hole-punch / TCP fallback — no more server involvement)
//! ```
//!
//! Pending requests expire after [`BROKER_TTL`] seconds if the peer does not
//! respond, preventing memory leaks from half-open handshakes.

use crate::control::Control;
use anyhow::{anyhow, Result};
use orbien_core::msg::{
    Message, P2pAddr, P2pInfo, P2pReady, P2pReq,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// How long a pending P2P request is kept before being garbage-collected.
pub const BROKER_TTL: Duration = Duration::from_secs(30);

/// State kept for one half of an in-progress P2P handshake.
struct PendingHalf {
    /// Arc to the initiating client's control connection so we can send it
    /// messages from the broker.
    control: Arc<Control>,
    /// Server-observed address of the initiator at the time of the request.
    observed_addr: SocketAddr,
    /// Candidate addresses reported by this side (set after P2pAddr arrives).
    candidates: Option<String>,
    /// When this entry was created — used for TTL eviction.
    created_at: Instant,
}

/// A pair of halves — initiator + responder — keyed by the shared broker token.
struct PendingPair {
    initiator: PendingHalf,
    /// Responder half is inserted when the peer accepts the request.
    responder: Option<PendingHalf>,
}

/// Shared broker state, protected by a single `Mutex`.
/// Contention is low: broker messages are rare compared to data traffic.
pub struct P2pBroker {
    pending: Mutex<HashMap<String, PendingPair>>,
}

impl P2pBroker {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pending: Mutex::new(HashMap::new()),
        })
    }

    // ── Step 1: initiator sends P2pReq ────────────────────────────────────────

    /// Called when the server receives a `P2pReq` on the initiator's control
    /// connection.
    ///
    /// 1. Looks up the responder's `Control` by `peer_session_id`.
    /// 2. Sends `P2pInfo` to the responder ("someone wants to punch through
    ///    to you").
    /// 3. Records the pending pair so we can complete the exchange when
    ///    `P2pAddr` messages arrive.
    pub async fn handle_req(
        &self,
        req: P2pReq,
        initiator_ctrl: Arc<Control>,
        initiator_peer: SocketAddr,
        responder_ctrl: Arc<Control>,
    ) -> Result<()> {
        let token = req.token.clone();

        // Evict stale entries before inserting a new one.
        self.evict_stale().await;

        // Tell the responder: "client A wants to connect directly to you".
        let info_to_responder = Message::P2pInfo(P2pInfo {
            token: token.clone(),
            peer_addr: initiator_peer.to_string(),
            error: String::new(),
        });
        responder_ctrl
            .send_p2p_msg(info_to_responder)
            .await
            .map_err(|e| anyhow!("send P2pInfo to responder: {e}"))?;

        // Also send P2pInfo to the initiator so it knows the broker accepted
        // the request (peer_addr will be filled in once the responder reports
        // its observed address via P2pAddr).
        let responder_addr = responder_ctrl.peer_addr();
        let info_to_initiator = Message::P2pInfo(P2pInfo {
            token: token.clone(),
            peer_addr: responder_addr.to_string(),
            error: String::new(),
        });
        initiator_ctrl
            .send_p2p_msg(info_to_initiator)
            .await
            .map_err(|e| anyhow!("send P2pInfo to initiator: {e}"))?;

        let mut map = self.pending.lock().await;
        map.insert(
            token,
            PendingPair {
                initiator: PendingHalf {
                    control: initiator_ctrl,
                    observed_addr: initiator_peer,
                    candidates: None,
                    created_at: Instant::now(),
                },
                responder: Some(PendingHalf {
                    control: responder_ctrl,
                    observed_addr: responder_addr,
                    candidates: None,
                    created_at: Instant::now(),
                }),
            },
        );
        Ok(())
    }

    // ── Step 2: a client sends P2pAddr ────────────────────────────────────────

    /// Called when a client sends a `P2pAddr` ("here are my candidate addrs").
    ///
    /// We store the candidates for this side.  When *both* sides have reported
    /// their candidates we send `P2pReady` to each and remove the entry.
    pub async fn handle_addr(
        &self,
        addr_msg: P2pAddr,
        from_session_id: &str,
    ) -> Result<()> {
        let token = &addr_msg.token;
        let mut map = self.pending.lock().await;

        let pair = map
            .get_mut(token)
            .ok_or_else(|| anyhow!("P2pAddr: unknown token {token}"))?;

        // Determine which side is sending.
        let from_initiator =
            pair.initiator.control.session_id() == from_session_id;

        if from_initiator {
            pair.initiator.candidates = Some(addr_msg.candidates.clone());
        } else if let Some(ref mut resp) = pair.responder {
            if resp.control.session_id() == from_session_id {
                resp.candidates = Some(addr_msg.candidates.clone());
            } else {
                return Err(anyhow!("P2pAddr from unknown session {from_session_id}"));
            }
        } else {
            return Err(anyhow!("P2pAddr: no responder for token {token}"));
        }

        // Check if both sides have now reported.
        let both_ready = pair.initiator.candidates.is_some()
            && pair
                .responder
                .as_ref()
                .map(|r| r.candidates.is_some())
                .unwrap_or(false);

        if both_ready {
            // Extract and remove.
            let pair = map.remove(token).unwrap();
            let resp = pair.responder.unwrap();

            let ready = P2pReady {
                token: token.clone(),
                initiator_candidates: pair.initiator.candidates.unwrap_or_default(),
                responder_candidates: resp.candidates.unwrap_or_default(),
                initiator_observed_addr: pair.initiator.observed_addr.to_string(),
                responder_observed_addr: resp.observed_addr.to_string(),
            };

            // Send P2pReady to both sides.  Failures are logged but do not
            // propagate — the handshake is best-effort once we get here.
            let msg_i = Message::P2pReady(ready.clone());
            let msg_r = Message::P2pReady(ready);

            let send_i = pair.initiator.control.send_p2p_msg(msg_i);
            let send_r = resp.control.send_p2p_msg(msg_r);
            let (ri, rr) = tokio::join!(send_i, send_r);
            if let Err(e) = ri {
                tracing::warn!(error = %e, "P2pReady send to initiator failed");
            }
            if let Err(e) = rr {
                tracing::warn!(error = %e, "P2pReady send to responder failed");
            }
        }
        Ok(())
    }

    // ── Housekeeping ──────────────────────────────────────────────────────────

    async fn evict_stale(&self) {
        let mut map = self.pending.lock().await;
        let before = map.len();
        map.retain(|_, pair| pair.initiator.created_at.elapsed() < BROKER_TTL);
        let evicted = before - map.len();
        if evicted > 0 {
            tracing::debug!(evicted, "evicted stale P2P broker entries");
        }
    }

    /// Number of pending (incomplete) P2P handshakes — for observability.
    #[allow(dead_code)]
    pub async fn pending_count(&self) -> usize {
        self.pending.lock().await.len()
    }
}

impl Default for P2pBroker {
    fn default() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn broker_ttl_constant_is_reasonable() {
        assert!(BROKER_TTL.as_secs() >= 10, "TTL too short");
        assert!(BROKER_TTL.as_secs() <= 120, "TTL too long");
    }

    #[tokio::test]
    async fn pending_count_starts_at_zero() {
        let broker = P2pBroker::new();
        assert_eq!(broker.pending_count().await, 0);
    }
}
