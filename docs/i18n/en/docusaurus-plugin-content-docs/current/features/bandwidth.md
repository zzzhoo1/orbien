---
sidebar_position: 4
sidebar_label: Bandwidth Limit
title: Bandwidth Limit
---

# Bandwidth Limit

Limit forwarded bandwidth per tunnel. Configure it under `transport` on `[[tunnels]]`.

- `bandwidthLimitSide = "client"`: limit on the client
- `bandwidthLimitSide = "server"`: limit on the server

`bandwidth` is a number in Mbps (e.g. `2`, `0.5`). `0` means unlimited.

## Example: Limit on the client

```toml
# orbien.toml
[[tunnels]]
name = "web"
protocol = "tcp"
service = "127.0.0.1:80"
remotePort = 9000
transport.bandwidth = 2
transport.bandwidthLimitSide = "client"
```

## Example: Limit on the server

```toml
# orbien.toml
[[tunnels]]
name = "web"
protocol = "tcp"
service = "127.0.0.1:80"
remotePort = 9000
transport.bandwidth = 0.5
transport.bandwidthLimitSide = "server"
```

## Parameters

| Parameter                        | Required | Default  | Description                               |
|----------------------------------|----------|----------|-------------------------------------------|
| `transport.bandwidth`            | No       | `0`      | Bandwidth cap (Mbps); `0` means unlimited |
| `transport.bandwidthLimitSide`   | No       | `client` | Limit side: `client` / `server`           |
