---
sidebar_position: 3
sidebar_label: QUIC
title: QUIC
---

# QUIC

UDP-based. Encryption and multiplexing are built in, so `tcpMux` / `transport.tls.enable` have no effect. Certificate verification can still be configured via `transport.tls`; see [TLS](../tls.md).

## Example

Server:

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"
quicPort = 9528

[transport.quic]
keepalivePeriod = 10
maxIdleTimeout = 30
maxIncomingStreams = 100000
```

Client:

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9528"

[transport]
protocol = "quic"

[transport.quic]
keepalivePeriod = 10
maxIdleTimeout = 30
maxIncomingStreams = 100000
```

`server` must point at the server `quicPort`.

## Client parameters

| Parameter | Required | Default | Description |
|------|------|--------|------|
| `transport.protocol` | Yes | `tcp` | Always `quic` |
| `server` | Yes | | Server address; the port is `quicPort` |
| `transport.quic.keepalivePeriod` | No | `10` | QUIC keepalive period (seconds) |
| `transport.quic.maxIdleTimeout` | No | `30` | QUIC idle timeout (seconds) |
| `transport.quic.maxIncomingStreams` | No | `100000` | Max concurrent bidirectional QUIC streams (also set on the client dial) |

## Server parameters

| Parameter | Required | Default | Description |
|------|------|--------|------|
| `quicPort` | Yes | `0` | QUIC listen port (UDP); `0` disables it; must not equal `kcpPort` |
| `transport.quic.keepalivePeriod` | No | `10` | QUIC keepalive period (seconds) |
| `transport.quic.maxIdleTimeout` | No | `30` | QUIC idle timeout (seconds) |
| `transport.quic.maxIncomingStreams` | No | `100000` | Max incoming QUIC streams |
