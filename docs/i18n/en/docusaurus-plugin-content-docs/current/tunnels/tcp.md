---
sidebar_position: 1
sidebar_label: TCP
title: TCP
---

# TCP

Map an internal TCP service to a public port on the server.

## Example: Expose SSH

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
name = "ssh"
protocol = "tcp"
service = "127.0.0.1:22"
remotePort = 9000
```

Access from the public network:

```shell
ssh -p 9000 user@YOUR_SERVER_IP
```

## Parameters

| Parameter                        | Required | Default     | Description                                      |
|----------------------------------|----------|-------------|--------------------------------------------------|
| `name`                           | Yes      |             | Tunnel name; must be unique                      |
| `protocol`                       | Yes      |             | Always `tcp`                                     |
| `service`                        | Yes      |             | Local service address, e.g. `127.0.0.1:22`       |
| `remotePort`                     | Yes      |             | Public listen port on the server                 |
| `transport.bandwidth`            | No       | `0`         | Bandwidth cap (Mbps); `0` means unlimited        |
| `transport.bandwidthLimitSide`   | No       | `client`    | Limit side: `client` / `server`                  |
| `transport.proxyProtocolVersion` | No       |             | PROXY Protocol: `v1` / `v2`                      |
