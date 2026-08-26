---
sidebar_position: 3
sidebar_label: TLS
title: TLS
---

# TLS

Control-channel TLS. Applies to `tcp` / `websocket` / `kcp` (QUIC is already encrypted, but the certificate path fields can still be used for QUIC identity verification).

Three modes:

1. **Encryption only**: Default; certificates are not verified
2. **Verify the server certificate**: The client trusts a CA and verifies the server identity
3. **Mutual TLS (mTLS)**: Both sides verify each other's certificates

The control channel uses a standard TLS ClientHello (first byte `0x16`). HTTP / HTTPS domain gateways must use separate ports; see [Domains](../tunnels/domains.md).

## Mode 1: Encryption only

Leave `trustedCaFile` unset; the client skips certificate verification. The server certificate may be omitted (a temporary self-signed cert is used).

Server:

```toml
# orbien-server.toml
[transport.tls]
force = false
```

Client:

```toml
# orbien.toml
[transport.tls]
enable = true
```

## Mode 2: Verify the server certificate

The server provides a certificate; the client verifies it with a CA and sets SNI.

Server:

```toml
# orbien-server.toml
[transport.tls]
force = true
certFile = "/path/to/server.crt"
keyFile = "/path/to/server.key"
```

Client:

```toml
# orbien.toml
[transport.tls]
enable = true
trustedCaFile = "/path/to/ca.crt"
serverName = "orbien.example.com"
```

## Mode 3: Mutual TLS (mTLS)

Both sides verify the peer certificate. Setting `trustedCaFile` on the server automatically implies `force = true`.

Server:

```toml
# orbien-server.toml
[transport.tls]
force = true
certFile = "/path/to/server.crt"
keyFile = "/path/to/server.key"
trustedCaFile = "/path/to/ca.crt"
```

Client:

```toml
# orbien.toml
[transport.tls]
enable = true
certFile = "/path/to/client.crt"
keyFile = "/path/to/client.key"
trustedCaFile = "/path/to/ca.crt"
serverName = "orbien.example.com"
```

## Client parameters

| Parameter                     | Required | Default | Description                                                          |
|-------------------------------|----------|---------|----------------------------------------------------------------------|
| `transport.tls.enable`        | No       | `true`  | Enable TLS; ignored for QUIC, which is already encrypted             |
| `transport.tls.certFile`      | No       |         | Client certificate (mTLS)                                            |
| `transport.tls.keyFile`       | No       |         | Client private key (mTLS)                                            |
| `transport.tls.trustedCaFile` | No       |         | Verify the server certificate; empty skips verification (encryption only) |
| `transport.tls.serverName`    | No       |         | TLS SNI; empty uses the host part of `server`                        |

## Server parameters

| Parameter                     | Required | Default | Description                                                          |
|-------------------------------|----------|---------|----------------------------------------------------------------------|
| `transport.tls.force`         | No       | `false` | Require TLS; reject non-TLS control connections                      |
| `transport.tls.certFile`      | No       |         | Server certificate; empty uses a temporary self-signed cert          |
| `transport.tls.keyFile`       | No       |         | Server private key; empty uses a temporary self-signed cert          |
| `transport.tls.trustedCaFile` | No       |         | Verify the client certificate (mTLS); non-empty automatically sets `force = true` |
