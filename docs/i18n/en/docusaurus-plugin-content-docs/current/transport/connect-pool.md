---
sidebar_position: 2
sidebar_label: Connection Pool
title: Connection Pool
---

# Connection Pool

Pre-create data connections/streams at login to reduce the latency of opening new connections. Client `poolCount` is capped by server `maxConnPool`.

## Example

Server:

```toml
# orbien-server.toml
[transport]
maxConnPool = 5
```

Client:

```toml
# orbien.toml
[transport]
poolCount = 3
```

The effective count is `min(poolCount, maxConnPool)`.

## Client parameters

| Parameter               | Required | Default | Description                                      |
|-------------------------|----------|---------|--------------------------------------------------|
| `transport.poolCount`   | No       | `1`     | Number of data connections/streams to pre-create at login |

## Server parameters

| Parameter                  | Required | Default | Description                                                      |
|----------------------------|----------|---------|------------------------------------------------------------------|
| `transport.maxConnPool`   | No       | `5`     | Cap on client `poolCount`; a configured `0` is automatically treated as `5` |
