# Orbien — Next-Phase Feature Roadmap

> Generated: 2026-09-01  
> Based on competitive analysis against **frp**, **rathole**, **bore**, and **NetsGo**.

---

## Priority Matrix

| # | Feature | Priority | Effort | Competitor Gap |
|---|---------|----------|--------|----------------|
| 1 | [Hot Reload](#1-hot-reload) | 🔴 High | Medium | frp ✅ rathole ✅ |
| 2 | [REST Management API](#2-rest-management-api) | 🔴 High | Medium | frp v2 ✅ NetsGo ✅ |
| 3 | [P2P Direct Tunnels](#3-p2p-direct-tunnels) | 🔴 High | High | frp xtcp ✅ NetsGo WebRTC ✅ |
| 4 | [Per-Tunnel Traffic Stats + Bandwidth Limiting](#4-per-tunnel-traffic-stats--bandwidth-limiting) | 🟡 Medium | Medium | frp Prometheus ✅ NetsGo ✅ |
| 5 | [Port Multiplexing / TCPMUX](#5-port-multiplexing--tcpmux) | 🟡 Medium | Medium | frp ✅ |
| 6 | [Tunnel Health Check & Auto-Reconnect Reporting](#6-tunnel-health-check--auto-reconnect-reporting) | 🟡 Medium | Low | frp ✅ |
| 7 | [OIDC / SSO Authentication](#7-oidc--sso-authentication) | 🟢 Low | Medium | frp ✅ |

---

## 1. Hot Reload

**Goal:** Apply config changes (add/remove/modify tunnels) without restarting the server or client process. Existing active connections must not be interrupted.

### Server-side
- Watch config file with [`notify`](https://crates.io/crates/notify) crate (cross-platform: inotify / kqueue / ReadDirectoryChangesW).
- On file change: re-parse config, diff against running state.
  - **Added tunnels** → start new listeners immediately.
  - **Removed tunnels** → gracefully drain; close listener after last active session exits.
  - **Modified params** (token, TLS, rate limits) → apply to new connections only.
- Expose `POST /api/v1/config/reload` so the Web UI / CI can trigger reload programmatically.
- Return a diff summary in the response body: `{ added: [...], removed: [...], modified: [...] }`.

### Client-side
- Same file-watch logic; reconnect only affected tunnels.
- `--watch` CLI flag to opt-in at startup.

### Web UI
- **Reload Config** button in the header / settings page.
- Toast notification on success/error with diff summary.
- New Pinia action `configStore.reload()` calling `POST /api/v1/config/reload`.

### Acceptance Criteria
- [ ] Add/remove a tunnel in `orbien-server.toml` → reflected within 1 s, no restart.
- [ ] Unchanged active connections survive the reload.
- [ ] `POST /api/v1/config/reload` returns HTTP 200 + diff JSON.
- [ ] Web UI button triggers the endpoint and displays the diff.
- [ ] Unit tests for diff logic; integration test for the full reload lifecycle.

---

## 2. REST Management API

**Goal:** A stable, versioned HTTP API (`/api/v1/`) for managing tunnels, clients, and server config programmatically — enabling CI/CD pipelines, Ansible playbooks, Homelab automation, and third-party dashboards.

### Endpoints (initial set)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/tunnels` | List all tunnels (paginated) |
| `GET` | `/api/v1/tunnels/:name` | Get tunnel detail + live stats |
| `POST` | `/api/v1/tunnels` | Create tunnel (hot-add) |
| `PUT` | `/api/v1/tunnels/:name` | Update tunnel config |
| `DELETE` | `/api/v1/tunnels/:name` | Remove tunnel (graceful) |
| `GET` | `/api/v1/clients` | List connected clients |
| `DELETE` | `/api/v1/clients/:id` | Disconnect client |
| `GET` | `/api/v1/system/info` | Server version, uptime, capabilities |
| `GET` | `/api/v1/system/stats` | Aggregate bandwidth + connection counts |
| `POST` | `/api/v1/config/reload` | Trigger hot reload (see §1) |

### Design Rules
- JSON request/response; `Content-Type: application/json`.
- Same session-cookie auth as the Web UI (`/api/v1/auth/*`).
- Paginated list responses: `{ data: [...], total: N, page: N, per_page: N }`.
- Error body: `{ error: "<code>", message: "<human>" }`.
- Versioned path prefix `/api/v1/` — future breaking changes go to `/api/v2/`.

### Web UI
- Update `server-ui/src/api/client.ts` to consume the new endpoints.
- Replace any remaining ad-hoc `fetch()` calls in views/stores with typed API helpers.

### Acceptance Criteria
- [ ] All endpoints above implemented and documented (OpenAPI spec in `doc/openapi.yaml`).
- [ ] Pagination works for `/api/v1/tunnels` and `/api/v1/clients`.
- [ ] Web UI migrated to use the typed API client exclusively.
- [ ] Integration tests for each endpoint (auth, CRUD, error cases).

---

## 3. P2P Direct Tunnels

**Goal:** Allow client-to-client traffic to flow directly without passing through the server, reducing latency and server bandwidth cost for high-throughput use cases (large file sync, video streams, etc.).

### Approach
- **STUN-based hole-punching** for symmetric/full-cone NAT scenarios (similar to frp `xtcp`).
- Server acts as signalling relay only; data flows P2P once the hole is punched.
- New tunnel type `p2p` in config:

```toml
[[tunnels]]
name = "big-file-sync"
protocol = "p2p"
service = "127.0.0.1:8080"
remotePort = 9100
```

- Fallback to relay mode automatically if hole-punch fails.
- Leverage existing QUIC/KCP transport layer where possible.

### Web UI
- Show P2P status badge on tunnel cards: `Direct` / `Relay` / `Negotiating`.
- Display RTT and packet-loss stats for P2P tunnels.

### Acceptance Criteria
- [ ] P2P tunnel establishes direct connection on same-NAT network.
- [ ] Auto-fallback to relay if STUN fails within configurable timeout.
- [ ] Status correctly reported as `Direct` or `Relay` in Web UI and REST API.
- [ ] Benchmark showing ≥ 2× throughput vs relay on LAN.

---

## 4. Per-Tunnel Traffic Stats & Bandwidth Limiting

**Goal:** Real-time per-tunnel byte/packet counters, historical graphs in the Web UI, and optional bandwidth caps per tunnel.

### Backend
- Atomic counters (`bytes_in`, `bytes_out`, `connections_total`, `connections_active`) per tunnel, updated in the data path.
- Expose via `GET /api/v1/tunnels/:name` (see §2).
- Optional: Prometheus `/metrics` endpoint (`orbien_tunnel_bytes_total{tunnel,direction}`).
- Bandwidth limiting: token-bucket per tunnel, configured as `rateLimit = "10Mbps"` in TOML.

### Web UI
- Sparkline charts on tunnel list cards (bytes/s, last 60 s).
- Detail page shows 1-hour history chart (ring buffer in memory, no DB required).
- Rate-limit badge on capped tunnels.

### Acceptance Criteria
- [ ] `bytes_in`/`bytes_out` counters accurate to within 1% under load.
- [ ] Prometheus metrics endpoint returns valid OpenMetrics text.
- [ ] Rate limiting enforced: client cannot exceed configured cap by more than 5%.
- [ ] Web UI sparklines update every 2 s.

---

## 5. Port Multiplexing / TCPMUX

**Goal:** Multiple services share a single external port via a virtual-host / name-based multiplexer, reducing the number of open ports needed on the server.

### HTTP Virtual Host
- Route by `Host` header → different backend tunnels.
- Config:

```toml
[[tunnels]]
name = "app-a"
protocol = "http"
service = "127.0.0.1:3000"
virtualHost = "app-a.example.com"

[[tunnels]]
name = "app-b"
protocol = "http"
service = "127.0.0.1:3001"
virtualHost = "app-b.example.com"
```

### TCP Multiplexing (TCPMUX)
- Prefix-based protocol for non-HTTP TCP services sharing one port.
- Clients use a connection header to identify the target tunnel.

### Acceptance Criteria
- [ ] Two HTTP tunnels on the same port routed correctly by `Host` header.
- [ ] TCPMUX protocol documented; client lib updated.
- [ ] Web UI shows `Virtual Host` field on HTTP tunnel detail pages.

---

## 6. Tunnel Health Check & Auto-Reconnect Reporting

**Goal:** Server-side health probes for each tunnel's backend service, with status visible in the Web UI and REST API. Client-side reconnect events surfaced transparently.

### Backend
- Optional per-tunnel health check config:

```toml
[[tunnels]]
name = "ssh"
protocol = "tcp"
service = "127.0.0.1:22"
healthCheck = { type = "tcp", intervalSecs = 10, timeoutSecs = 3, unhealthyThreshold = 2 }
```

- Health states: `healthy` / `unhealthy` / `unknown`.
- Unhealthy tunnels rejected at the proxy layer (return 503 for HTTP; reset TCP for raw).

### Client-side
- Emit reconnect events with timestamp, attempt count, and reason.
- REST API: `GET /api/v1/clients/:id/reconnects` returns reconnect history (last 100).

### Web UI
- Health indicator dot (green/red/gray) on each tunnel card.
- Reconnect count badge on client cards.

### Acceptance Criteria
- [ ] TCP health check correctly marks tunnel unhealthy when backend is down.
- [ ] Unhealthy HTTP tunnels return 503 to the caller.
- [ ] Reconnect history available via REST API and visible in Web UI.

---

## 7. OIDC / SSO Authentication

**Goal:** Support OpenID Connect for enterprise SSO (Google Workspace, Okta, Authentik, etc.) in addition to the existing token + WebAuthn auth.

### Backend
- New auth method `oidc` in server config:

```toml
[auth]
method = "oidc"
oidc.issuerUrl = "https://accounts.google.com"
oidc.clientId = "..."
oidc.clientSecret = "..."
oidc.redirectUrl = "https://my-server/api/v1/auth/oidc/callback"
```

- Standard Authorization Code flow; session cookie issued on success.
- Optional allowed-domains list: `oidc.allowedDomains = ["mycompany.com"]`.

### Web UI
- Login page detects `oidc` capability (already wired via `capabilities` in `useAuthStore`) and shows **Sign in with SSO** button.
- Uses existing `capabilitiesLoaded` / `loadCapabilities()` pattern.

### Acceptance Criteria
- [ ] Full OIDC Authorization Code flow works end-to-end with at least one provider (Google or Authentik).
- [ ] `oidc.allowedDomains` filter enforced server-side.
- [ ] Login page shows SSO button only when capability is reported.
- [ ] Existing token + WebAuthn auth unaffected.

---

## Implementation Order

```
Phase 1 (next release)
  └── §2 REST API          ← foundation; §1 and §4 depend on it
  └── §1 Hot Reload        ← uses POST /api/v1/config/reload from §2
  └── §6 Health Check      ← low effort, high visibility

Phase 2
  └── §4 Traffic Stats     ← builds on REST API
  └── §5 Port Multiplexing

Phase 3
  └── §3 P2P Direct        ← high effort; needs §2 for signalling API
  └── §7 OIDC              ← enterprise tier
```
