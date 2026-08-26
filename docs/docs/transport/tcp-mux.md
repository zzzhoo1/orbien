---
sidebar_position: 2.5
sidebar_label: TCP 多路复用
title: TCP 多路复用
---

# TCP 多路复用

一条物理连接承载多路逻辑流（Yamux），降低建连开销。适用于 `tcp` / `websocket` / `kcp`；**QUIC 自带多路复用，此项无效**。

客户端与服务端 `tcpMux` **必须一致**，否则无法握手。

## 示例：开启

TCP 多路复用默认开启。开启后应用层心跳默认可关闭（`heartbeatInterval` / `heartbeatTimeout` = `-1`），改由多路复用保活检测死连接。

服务端：

```toml
# orbien-server.toml
[transport]
tcpMux = true
muxKeepaliveSecs = 30
```

客户端：

```toml
# orbien.toml
[transport]
tcpMux = true
muxKeepaliveSecs = 30
```

## 示例：关闭

关闭后每条控制/数据流使用独立连接，需开启应用层心跳。

服务端：

```toml
# orbien-server.toml
[transport]
tcpMux = false
heartbeatTimeout = 90
```

客户端：

```toml
# orbien.toml
[transport]
tcpMux = false
heartbeatInterval = 30
heartbeatTimeout = 90
```

客户端若省略心跳相关项，关闭 `tcpMux` 时会自动设为 `heartbeatInterval=30`、`heartbeatTimeout=90`。

## 客户端参数

| 参数 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `transport.tcpMux` | 否 | `true` | 是否启用多路复用；需与服务端一致 |
| `transport.muxKeepaliveSecs` | 否 | `30` | 多路复用保活间隔（秒）。Rust yamux 无原生 KeepAlive，实现上在 `tcpMux=true` 且未开应用心跳时，用该间隔发控制面 Ping（等效保活） |
| `transport.heartbeatInterval` | 否 | `-1`（mux 开）/ `30`（mux 关） | 应用心跳间隔（秒）；`-1` 关闭 |
| `transport.heartbeatTimeout` | 否 | `-1`（mux 开）/ `90`（mux 关） | 超过该秒数未收到 Pong 则断开；`-1` 关闭 |

## 服务端参数

| 参数 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `transport.tcpMux` | 否 | `true` | 是否启用多路复用；需与客户端一致 |
| `transport.muxKeepaliveSecs` | 否 | `30` | 多路复用保活间隔（秒）。Rust yamux 无原生 KeepAlive；在 `tcpMux=true` 且未开应用心跳超时时，用 `间隔×3` 作为「未收到客户端 Ping 则踢掉」的超时（与客户端对称） |
| `transport.heartbeatTimeout` | 否 | `-1`（mux 开）/ `90`（mux 关） | 超过该秒数未收到客户端 Ping 则踢掉控制连接；`-1` 表示关闭应用心跳超时（mux 开启时改走上方保活逻辑） |
