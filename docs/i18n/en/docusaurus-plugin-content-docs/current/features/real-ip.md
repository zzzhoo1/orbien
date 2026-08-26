---
sidebar_position: 5
sidebar_label: Real IP
title: Real IP
---

# Real IP

After tunneling, the local service usually sees the tunnel-side address. Use the following to get the visitor IP.

## PROXY Protocol

The client writes a PROXY Protocol header when connecting to the local service. The local service must support parsing it (e.g. Nginx, HAProxy).

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9527"

[[tunnels]]
name = "web"
protocol = "tcp"
service = "127.0.0.1:80"
remotePort = 9000
transport.proxyProtocolVersion = "v2"
```

Applies to `tcp` / `udp` / `http` / `https` (passthrough). Not available with `tls-term`.

| Parameter                        | Required | Default | Description                          |
|----------------------------------|----------|---------|--------------------------------------|
| `transport.proxyProtocolVersion` | No       |         | `v1` / `v2`; empty disables it       |

## X-Forwarded-For

Injected automatically for `http` on the server, and for `https` + `tls-term` by the client plugin:

- `X-Forwarded-For`: visitor IP
- `X-Forwarded-Proto`: `http` or `https`

Read these headers in your app.

## Note

The server does not parse PROXY Protocol from an upstream load balancer. The visitor source is the TCP peer that connects to `proxyAddr`.
