---
sidebar_position: 2
sidebar_label: UDP
title: UDP
---

# UDP

Map an internal UDP service to a public port on the server.

## Example: Expose DNS

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
name = "dns"
protocol = "udp"
service = "127.0.0.1:53"
remotePort = 9000
```

Access from the public network:

```shell
dig @YOUR_SERVER_IP -p 9000 example.com
```

## Parameters

| Parameter                        | Required | Default     | Description                                      |
|----------------------------------|----------|-------------|--------------------------------------------------|
| `udpPacketSize`                  | No       | `1500`      | Max UDP datagram size; must match the server     |
| `name`                           | Yes      |             | Tunnel name; must be unique                      |
| `protocol`                       | Yes      |             | Always `udp`                                     |
| `service`                        | Yes      |             | Local service address, e.g. `127.0.0.1:53`       |
| `remotePort`                     | Yes      |             | Public listen port on the server                 |
| `transport.bandwidth`            | No       | `0`         | Bandwidth cap (Mbps); `0` means unlimited        |
| `transport.bandwidthLimitSide`   | No       | `client`    | Limit side: `client` / `server`                  |
| `transport.proxyProtocolVersion` | No       |             | PROXY Protocol: `v1` / `v2`                      |
