---
sidebar_position: 3
sidebar_label: HTTP
title: HTTP
---

# HTTP

Expose an internal HTTP service to the public internet by domain. The server must enable the domain gateway; see [Domains](./domains.md).

`domains` is required and must be non-empty: a full domain (e.g. `web.example.com`), or a short prefix with no `.` (e.g. `web`, which requires server `rootDomain`). Wildcards are not supported.

## Example: Expose a web service

Server:

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"
httpGwPort = 80
```

Client:

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9527"

[[tunnels]]
name = "web"
protocol = "http"
service = "127.0.0.1:8080"
domains = ["web.example.com"]
```

Point `web.example.com` at the server IP, then access:

```shell
curl http://web.example.com
```

## Example: Basic authentication

```toml
[[tunnels]]
name = "web-auth"
protocol = "http"
service = "127.0.0.1:8080"
domains = ["web.example.com"]
basicAuthUser = "alice"
basicAuthPassword = "secret"
```

Requests must include Basic credentials; failure returns `401`:

```shell
curl -u alice:secret http://web.example.com
```

## Example: Route by HTTP user

On the same domain, different Basic users can be routed to different local services. If no dedicated route matches, traffic falls back to a tunnel without `routeByHTTPUser` (if one exists).

```toml
[[tunnels]]
name = "web-alice"
protocol = "http"
service = "127.0.0.1:8081"
domains = ["web.example.com"]
routeByHTTPUser = "alice"
basicAuthUser = "alice"
basicAuthPassword = "secret"

[[tunnels]]
name = "web-default"
protocol = "http"
service = "127.0.0.1:8080"
domains = ["web.example.com"]
```

## Example: Rewrite Host

Rewrite the request `Host` when forwarding to the local service:

```toml
[[tunnels]]
name = "web"
protocol = "http"
service = "127.0.0.1:8080"
domains = ["web.example.com"]
hostHeaderRewrite = "127.0.0.1"
```

## Parameters

| Parameter                        | Required | Default     | Description                                                                                      |
|----------------------------------|----------|-------------|--------------------------------------------------------------------------------------------------|
| `name`                           | Yes      |             | Tunnel name; must be unique                                                                      |
| `protocol`                       | Yes      |             | Always `http`                                                                                    |
| `service`                        | Yes      |             | Local service address, e.g. `127.0.0.1:8080`                                                     |
| `domains`                        | Yes      |             | Domain list; at least one; full domain or a prefix with no `.` (prefix requires server `rootDomain`) |
| `locations`                      | No       |             | Path prefix, e.g. `/api`; empty means all paths                                                  |
| `basicAuthUser`                  | No       |             | HTTP Basic username; if both this and `basicAuthPassword` are empty, auth is disabled            |
| `basicAuthPassword`              | No       |             | HTTP Basic password                                                                              |
| `routeByHTTPUser`                | No       |             | Select a route by the request Basic username; empty matches any user (exact user takes priority) |
| `hostHeaderRewrite`              | No       |             | Rewrite Host when forwarding to the local service; empty means no rewrite                        |
| `transport.bandwidth`            | No       | `0`         | Bandwidth cap (Mbps); `0` means unlimited                                                        |
| `transport.bandwidthLimitSide`   | No       | `client`    | Limit side: `client` / `server`                                                                  |
| `transport.proxyProtocolVersion` | No       |             | PROXY Protocol: `v1` / `v2`                                                                      |
