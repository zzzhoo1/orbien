---
sidebar_position: 4
sidebar_label: HTTPS
title: HTTPS
---

# HTTPS

Expose an internal service to the public internet by domain. The server must enable the domain gateway; see [Domains](./domains.md).

`domains` is required and must be non-empty: a full domain (e.g. `web.example.com`), or a short prefix with no `.` (e.g. `web`, which requires server `rootDomain`). Wildcards are not supported.

Two modes:

- **Passthrough**: Forward by SNI; the certificate is on the internal HTTPS service
- **TLS termination**: The client plugin `tls-term` terminates TLS, then forwards to a local HTTP service

## Example: Passthrough

Server:

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"
httpsGwPort = 443
```

Client:

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9527"

[[tunnels]]
name = "web-ssl"
protocol = "https"
service = "127.0.0.1:443"
domains = ["web.example.com"]
```

Point `web.example.com` at the server IP, then access:

```shell
curl https://web.example.com
```

## Example: TLS termination

The client terminates TLS; the backend only needs to serve HTTP:

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9527"

[[tunnels]]
name = "https-term"
protocol = "https"
domains = ["web.example.com"]

[tunnels.plugin]
type = "tls-term"
service = "127.0.0.1:80"
certFile = "/path/to/cert.pem"
keyFile = "/path/to/key.pem"
```

`certFile` / `keyFile` may be omitted. If omitted, a temporary self-signed certificate is used (browsers will warn that it is untrusted). When the plugin is enabled, use `plugin.service` to point at local HTTP; the tunnel-level `service` is not used, and PROXY Protocol cannot be configured.

## Example: Rewrite Host with TLS termination

Under `tls-term`, rewrite `Host` before forwarding to local HTTP:

```toml
[[tunnels]]
name = "https-term"
protocol = "https"
domains = ["web.example.com"]

[tunnels.plugin]
type = "tls-term"
service = "127.0.0.1:80"
certFile = "/path/to/cert.pem"
keyFile = "/path/to/key.pem"
hostHeaderRewrite = "127.0.0.1"
```

## Example: Add request headers with TLS termination

Under `tls-term`, append custom request headers to local HTTP:

```toml
[[tunnels]]
name = "https-term"
protocol = "https"
domains = ["web.example.com"]

[tunnels.plugin]
type = "tls-term"
service = "127.0.0.1:80"
certFile = "/path/to/cert.pem"
keyFile = "/path/to/key.pem"

[tunnels.plugin.requestHeaders.set]
X-From = "orbien"
```

## Parameters

| Parameter                        | Required    | Default     | Description                                                                                      |
|----------------------------------|-------------|-------------|--------------------------------------------------------------------------------------------------|
| `name`                           | Yes         |             | Tunnel name; must be unique                                                                      |
| `protocol`                       | Yes         |             | Always `https`                                                                                   |
| `service`                        | Conditional |             | Local service address (required for passthrough), e.g. `127.0.0.1:443`                           |
| `domains`                        | Yes         |             | Domain list; at least one; full domain or a prefix with no `.` (prefix requires server `rootDomain`) |
| `plugin.type`                    | No          |             | `tls-term`: terminate TLS on the client                                                          |
| `plugin.service`                 | Conditional |             | Required for `tls-term`, e.g. `127.0.0.1:80`                                                     |
| `plugin.certFile`                 | No          |             | Certificate path; empty uses a temporary self-signed cert                                        |
| `plugin.keyFile`                 | No          |             | Private key path; empty uses a temporary self-signed cert                                        |
| `plugin.hostHeaderRewrite`       | No          |             | Rewrite Host when forwarding to the local service; empty means no rewrite                        |
| `plugin.requestHeaders.set`      | No          |             | Extra request headers to send to the backend; key-value pairs                                    |
| `transport.bandwidth`            | No          | `0`         | Bandwidth cap (Mbps); `0` means unlimited                                                        |
| `transport.bandwidthLimitSide`   | No          | `client`    | Limit side: `client` / `server`                                                                  |
| `transport.proxyProtocolVersion` | No          |             | PROXY Protocol: `v1` / `v2` (not available with `tls-term`)                                      |
