---
sidebar_position: 5
sidebar_label: SOCKS5
title: SOCKS5
---

# SOCKS5

Expose a public port on the server as a SOCKS5 proxy entry; the client performs the handshake and dialing on the
internal network.

Based on `protocol = "tcp"`, enabled via `[tunnels.plugin]`; both `username` and `password` are required.

## Example

Server:

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"
```

Client:

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9527"

[[tunnels]]
name = "socks5"
protocol = "tcp"
remotePort = 9000

[tunnels.plugin]
type = "socks5"
username = "admin"
password = "123456"
```

Access through the proxy from the public network:

```shell
curl --socks5 YOUR_SERVER_IP:9000 -U admin:123456 http://example.com
```

## Parameters

| Parameter                        | Required | Default  | Description                                     |
|----------------------------------|----------|----------|-------------------------------------------------|
| `name`                           | Yes      |          | Tunnel name; must be unique                     |
| `protocol`                       | Yes      |          | Always `tcp`                                    |
| `remotePort`                     | Yes      |          | Public listen port on the server (SOCKS5 entry) |
| `plugin.type`                    | Yes      |          | Always `socks5`                                 |
| `plugin.username`                | Yes      |          | SOCKS5 username                                 |
| `plugin.password`                | Yes      |          | SOCKS5 password                                 |
| `transport.bandwidth`            | No       | `0`      | Bandwidth cap (Mbps); `0` means unlimited       |
| `transport.bandwidthLimitSide`   | No       | `client` | Limit side: `client` / `server`                 |
| `transport.proxyProtocolVersion` | No       |          | PROXY Protocol: `v1` / `v2`                     |
