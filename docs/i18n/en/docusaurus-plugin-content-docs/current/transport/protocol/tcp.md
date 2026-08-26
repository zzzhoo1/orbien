---
sidebar_position: 1
sidebar_label: TCP
title: TCP
---

# TCP

The default transport protocol. The client establishes a control channel through the server `listen` address. For TLS, see [TLS](../tls.md).

## Example

Server:

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"

[transport]
tcpMux = true
```

Client:

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9527"

[transport]
protocol = "tcp"
tcpMux = true
heartbeatInterval = -1
```

## Client parameters

| Parameter                     | Required | Default | Description                                                                                          |
|-------------------------------|----------|---------|------------------------------------------------------------------------------------------------------|
| `transport.protocol`          | No       | `tcp`   | Always `tcp`                                                                                         |
| `transport.tcpMux`            | No       | `true`  | TCP multiplexing; must match the server                                                              |
| `transport.heartbeatInterval` | No       | `-1`    | Application heartbeat interval (seconds); `-1` disables it. Becomes `30` when `tcpMux` is off        |
| `transport.heartbeatTimeout`  | No       | `-1`    | Heartbeat timeout (seconds); disconnect if no Pong. Becomes `90` when `tcpMux` is off; see [TCP Multiplexing](../tcp-mux.md) |

## Server parameters

| Parameter          | Required | Default          | Description                          |
|--------------------|----------|------------------|--------------------------------------|
| `listen`           | Yes      | `0.0.0.0:9527`   | TCP listen address                   |
| `transport.tcpMux` | No       | `true`           | TCP multiplexing; must match the client |
