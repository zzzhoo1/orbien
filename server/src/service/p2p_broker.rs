//! Server-side P2P broker — coordinates the address-exchange handshake
//! between two orbien clients that want to establish a direct tunnel.
//!
//! # Protocol flow
//!
//! ```text
//! Client A (initiator)          Broker                  Client B (responder)
//!   │                              │                              │
//!   │── P2pReq {peer=B, token,     │                              │
//!   │           tunnel_name} ─────>│                              │
//!   │                              │── P2pInfo {token, peer_addr=A_obs} ──>│
//!   │<─ P2pInfo {token, peer_addr=B_obs} ──────────────────────────│
//!   │── P2pAddr {token, cands_A} ─>│                              │
//!   │                              │<─ P2pAddr {token, cands_B} ──│
//!   │<─ P2pReady {tunnel_name} ────│── P2pReady {tunnel_name} ───>│
//!   │                              │                              │
//!   │  (UDP hole-punch / TCP connect — no more server involvement)
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
    control: Arc<Control>,
    observed_addr: SocketAddr,
    candidates: Option<String>,
    created_at: Instant,
}

/// A pair of halves — initiator + responder — keyed by the shared broker token.
struct PendingPair {
    initiator: PendingHalf,
    responder: Option<PendingHalf>,
    /// Tunnel name supplied by the initiator in `P2pReq`; echoed in
    /// `P2pReady` to both sides so each client knows which backend to dial.
    /// Empty when the initiator is an old client that did not send the field.
    tunnel_name: String,
}

/// Shared broker state, protected by a single `Mutex`.
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

    pub async fn handle_req(
        &self,
        req: P2pReq,
        initiator_ctrl: Arc<Control>,
        initiator_peer: SocketAddr,
        responder_ctrl: Arc<Control>,
    ) -> Result<()> {
        let token = req.token.clone();
        let tunnel_name = req.tunnel_name.clone();

        self.evict_stale().await;

        let responder_addr = responder_ctrl.peer_addr();

        // Build both messages before taking the lock.
        let info_to_responder = Message::P2pInfo(P2pInfo {
            token: token.clone(),
            peer_addr: initiator_peer.to_string(),
            error: String::new(),
        });
        let info_to_initiator = Message::P2pInfo(P2pInfo {
            token: token.clone(),
            peer_addr: responder_addr.to_string(),
            error: String::new(),
        });

        // Insert into the map first, then send — keeps lock scope minimal.
        {
            let mut map = self.pending.lock().await;
            map.insert(
                token.clone(),
                PendingPair {
                    initiator: PendingHalf {
                        control: Arc::clone(&initiator_ctrl),
                        observed_addr: initiator_peer,
                        candidates: None,
                        created_at: Instant::now(),
                    },
                    responder: Some(PendingHalf {
                        control: Arc::clone(&responder_ctrl),
                        observed_addr: responder_addr,
                        candidates: None,
                        created_at: Instant::now(),
                    }),
                    tunnel_name,
                },
            );
        } // lock released here — I/O below does NOT hold the mutex

        responder_ctrl
            .send_p2p_msg(info_to_responder)
            .await
            .map_err(|e| anyhow!("send P2pInfo to responder: {e}"))?;

        initiator_ctrl
            .send_p2p_msg(info_to_initiator)
            .await
            .map_err(|e| anyhow!("send P2pInfo to initiator: {e}"))?;

        Ok(())
    }

    // ── Step 2: a client sends P2pAddr ────────────────────────────────────────

    pub async fn handle_addr(
        &self,
        addr_msg: P2pAddr,
        from_session_id: &str,
    ) -> Result<()> {
        let token = addr_msg.token.clone();

        // --- critical section: read/update map state only, no I/O ---
        let ready_pair: Option<(Arc<Control>, Arc<Control>, P2pReady)> = {
            let mut map = self.pending.lock().await;

            let pair = map
                .get_mut(&token)
                .ok_or_else(|| anyhow!("P2pAddr: unknown token {token}"))?;

            let from_initiator = pair.initiator.control.session_id() == from_session_id;

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

            let both_ready = pair.initiator.candidates.is_some()
                && pair
                    .responder
                    .as_ref()
                    .map(|r| r.candidates.is_some())
                    .unwrap_or(false);

            if both_ready {
                let pair = map.remove(&token).unwrap();
                let resp = pair.responder.unwrap();
                let ready = P2pReady {
                    token: token.clone(),
                    initiator_candidates: pair.initiator.candidates.unwrap_or_default(),
                    responder_candidates: resp.candidates.unwrap_or_default(),
                    initiator_observed_addr: pair.initiator.observed_addr.to_string(),
                    responder_observed_addr: resp.observed_addr.to_string(),
                    tunnel_name: pair.tunnel_name,
                };
                Some((pair.initiator.control, resp.control, ready))
            } else {
                None
            }
        }; // lock released here

        // --- I/O outside the lock ---
        if let Some((initiator_ctrl, responder_ctrl, ready)) = ready_pair {
            let msg_i = Message::P2pReady(ready.clone());
            let msg_r = Message::P2pReady(ready);
            let (ri, rr) = tokio::join!(
                initiator_ctrl.send_p2p_msg(msg_i),
                responder_ctrl.send_p2p_msg(msg_r),
            );
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
    use orbien_core::msg::P2pReady;

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

    /// The tunnel_name supplied by the initiator must appear verbatim in the
    /// P2pReady that is sent to both sides.  This mirrors the assignment in
    /// `handle_addr`: `tunnel_name: pair.tunnel_name`.
    #[test]
    fn p2p_ready_carries_initiator_tunnel_name() {
        let ready = P2pReady {
            token: "tok-1".into(),
            initiator_candidates: "10.0.0.1:1111,203.0.113.1:2222".into(),
            responder_candidates: "10.0.0.2:3333,198.51.100.2:4444".into(),
            initiator_observed_addr: "203.0.113.1:2222".into(),
            responder_observed_addr: "198.51.100.2:4444".into(),
            tunnel_name: "tcp-demo".into(),
        };

        assert_eq!(
            ready.tunnel_name, "tcp-demo",
            "tunnel_name must be preserved from initiator into P2pReady"
        );
    }

    /// When the initiator sends a non-empty tunnel_name, a different value
    /// that might be associated with the responder side must NOT end up in
    /// the ready message — the initiator's value is canonical.
    #[test]
    fn responder_value_does_not_override_initiator_tunnel_name() {
        let initiator_tunnel_name = "initiator-tunnel";
        let responder_side_different_name = "responder-tunnel";

        // Simulate the broker's assignment: `tunnel_name: pair.tunnel_name`
        // where `pair.tunnel_name` was set from the initiator's P2pReq.
        let ready = P2pReady {
            token: "tok-2".into(),
            initiator_candidates: "10.0.0.1:1111".into(),
            responder_candidates: "10.0.0.2:2222".into(),
            initiator_observed_addr: "203.0.113.1:1111".into(),
            responder_observed_addr: "198.51.100.2:2222".into(),
            tunnel_name: initiator_tunnel_name.into(),
        };

        assert_eq!(ready.tunnel_name, initiator_tunnel_name);
        assert_ne!(
            ready.tunnel_name, responder_side_different_name,
            "responder-side name must not override initiator tunnel_name"
        );
    }

    /// An old initiator that omits tunnel_name (empty string) must not cause
    /// a panic or unexpected value — both sides fall back to relay mode when
    /// they receive an empty tunnel_name in P2pReady.
    #[test]
    fn empty_tunnel_name_is_preserved_as_empty() {
        let ready = P2pReady {
            token: "tok-3".into(),
            initiator_candidates: "10.0.0.1:1111".into(),
            responder_candidates: "10.0.0.2:2222".into(),
            initiator_observed_addr: "203.0.113.1:1111".into(),
            responder_observed_addr: "198.51.100.2:2222".into(),
            tunnel_name: String::new(), // old client — no tunnel_name
        };

        assert!(
            ready.tunnel_name.is_empty(),
            "empty tunnel_name must stay empty (relay fallback sentinel)"
        );
    }
}
