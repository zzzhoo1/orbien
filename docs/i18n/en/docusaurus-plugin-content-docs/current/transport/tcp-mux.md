---
sidebar_position: 2.5
sidebar_label: TCP Multiplexing
title: TCP Multiplexing
---

# TCP Multiplexing

One physical connection carries multiple logical streams (Yamux), reducing connection-setup overhead. Applies to `tcp` / `websocket` / `kcp`; **QUIC has built-in multiplexing, so this setting has no effect**.

Client and server `tcpMux` **must match**, or the handshake will fail.

## Example: Enable

TCP multiplexing is enabled by default. When it is on, application heartbeats can stay off (`heartbeatInterval` / `heartbeatTimeout` = `-1`); dead connections are detected by multiplexing keepalive instead.

Server:

```toml
# orbien-server.toml
[transport]
tcpMux = true
muxKeepaliveSecs = 30
```

Client:

```toml
# orbien.toml
[transport]
tcpMux = true
muxKeepaliveSecs = 30
```

## Example: Disable

When disabled, each control/data stream uses its own connection, so application heartbeats should be enabled.

Server:

```toml
# orbien-server.toml
[transport]
tcpMux = false
heartbeatTimeout = 90
```

Client:

```toml
# orbien.toml
[transport]
tcpMux = false
heartbeatInterval = 30
heartbeatTimeout = 90
```

If the client omits heartbeat settings, disabling `tcpMux` automatically sets `heartbeatInterval=30` and `heartbeatTimeout=90`.

## Client parameters

| Parameter | Required | Default | Description |
|------|------|--------|------|
| `transport.tcpMux` | No | `true` | Enable multiplexing; must match the server |
| `transport.muxKeepaliveSecs` | No | `30` | Multiplexing keepalive interval (seconds). Rust yamux has no native KeepAlive; when `tcpMux=true` and application heartbeats are off, a control-plane Ping is sent at this interval (equivalent keepalive) |
| `transport.heartbeatInterval` | No | `-1` (mux on) / `30` (mux off) | Application heartbeat interval (seconds); `-1` disables it |
| `transport.heartbeatTimeout` | No | `-1` (mux on) / `90` (mux off) | Disconnect if no Pong is received within this many seconds; `-1` disables it |

## Server parameters

| Parameter | Required | Default | Description |
|------|------|--------|------|
| `transport.tcpMux` | No | `true` | Enable multiplexing; must match the client |
| `transport.muxKeepaliveSecs` | No | `30` | Multiplexing keepalive interval (seconds). Rust yamux has no native KeepAlive; when `tcpMux=true` and application heartbeat timeout is off, `interval×3` is used as the timeout for kicking a client that sends no Ping (symmetric with the client) |
| `transport.heartbeatTimeout` | No | `-1` (mux on) / `90` (mux off) | Kick the control connection if no client Ping is received within this many seconds; `-1` disables application heartbeat timeout (when mux is on, the keepalive logic above is used instead) |
