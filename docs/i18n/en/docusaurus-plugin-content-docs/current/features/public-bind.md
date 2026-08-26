---
sidebar_position: 3
sidebar_label: Public Bind Address
title: Public Bind Address
---

# Public Bind Address

The server has two kinds of listeners:

| Purpose          | Config                                                | Who connects                                              |
|------------------|-------------------------------------------------------|-----------------------------------------------------------|
| Control channel  | `listen` (and `quicPort` / `kcpPort`)                 | The **client** (`orbien`) connects to build tunnels       |
| Public traffic   | `proxyAddr` + `remotePort` / `httpGwPort` / `httpsGwPort` | **Public visitors** access the services you exposed     |

`proxyAddr` chooses which NIC / address visitor traffic binds to. It is independent of where the client connects. The config key remains `proxyAddr` (historical name); it means the **public bind address**”.

Typical case: on a multi-homed host, bind the control channel to the private network and public listeners to the public IP — or the reverse, to isolate management from traffic.

Default is `0.0.0.0` (all interfaces). If empty, it falls back to the host part of `listen`.

## Example

Control channel and public listeners on all interfaces (default):

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"
proxyAddr = "0.0.0.0"
```

Serve public traffic only on the public NIC (example IP):

```toml
# orbien-server.toml
listen = "10.0.0.2:9527"
proxyAddr = "203.0.113.10"
```

The client then connects to `10.0.0.2:9527`; visitors reach `remotePort` / `httpGwPort` / `httpsGwPort` on `203.0.113.10`.

## Parameters

| Parameter   | Required | Default     | Description                                              |
|-------------|----------|-------------|----------------------------------------------------------|
| `proxyAddr` | No       | `0.0.0.0`   | Public bind address; empty falls back to the host part of `listen` |
