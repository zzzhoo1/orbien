---
sidebar_position: 4
sidebar_label: KCP
title: KCP
---

# KCP

Reliable transport over UDP; more stable on lossy networks. For TLS, see [TLS](../tls.md).

## Example

Server:

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"
kcpPort = 9529

[transport]
tcpMux = true
```

Client:

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9529"

[transport]
protocol = "kcp"
tcpMux = true
heartbeatInterval = -1
```

`server` must point at the server `kcpPort`, and it must not equal `quicPort`.

## Client parameters

| Parameter                     | Required | Default | Description                                                                                          |
|-------------------------------|----------|---------|------------------------------------------------------------------------------------------------------|
| `transport.protocol`          | Yes      | `tcp`   | Always `kcp`                                                                                         |
| `server`                      | Yes      |         | Server address; the port is `kcpPort`                                                                |
| `transport.tcpMux`            | No       | `true`  | TCP multiplexing; must match the server                                                              |
| `transport.heartbeatInterval` | No       | `-1`    | Application heartbeat interval (seconds); `-1` disables it. Becomes `30` when `tcpMux` is off        |
| `transport.heartbeatTimeout`  | No       | `-1`    | Heartbeat timeout (seconds); disconnect if no Pong. Becomes `90` when `tcpMux` is off; see [TCP Multiplexing](../tcp-mux.md) |

## Server parameters

| Parameter          | Required | Default | Description                                                      |
|--------------------|----------|---------|------------------------------------------------------------------|
| `kcpPort`          | Yes      | `0`     | KCP listen port (UDP); `0` disables it; must not equal `quicPort` |
| `transport.tcpMux` | No       | `true`  | TCP multiplexing; must match the client                          |
