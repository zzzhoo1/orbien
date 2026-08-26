---
sidebar_position: 5
sidebar_label: SOCKS5
title: SOCKS5
---

# SOCKS5

将服务端公网端口作为 SOCKS5 代理入口，由客户端在内网完成握手与拨号。

基于 `protocol = "tcp"`，通过 `[tunnels.plugin]` 启用；须同时配置 `username` 与 `password`。

## 示例

服务端：

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"
```

客户端：

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9527"

[[tunnels]]
name = "socks5"
protocol = "tcp"
remotePort = 9000

[tunnels.plugin]
type = "socks5"
username = "admin"
password = "123456"
```

公网通过代理访问：

```shell
curl --socks5 YOUR_SERVER_IP:9000 -U admin:123456 http://example.com
```

## 参数

| 参数                               | 必填 | 默认值      | 说明                         |
|----------------------------------|----|----------|----------------------------|
| `name`                           | 是  |          | 隧道名称，唯一                    |
| `protocol`                       | 是  |          | 固定为 `tcp`                  |
| `remotePort`                     | 是  |          | 服务端对外监听端口（SOCKS5 入口）       |
| `plugin.type`                    | 是  |          | 固定为 `socks5`               |
| `plugin.username`                | 是  |          | SOCKS5 用户名                 |
| `plugin.password`                | 是  |          | SOCKS5 密码                  |
| `transport.bandwidth`            | 否  | `0`      | 带宽上限（Mbps）；`0` 表示不限制       |
| `transport.bandwidthLimitSide`   | 否  | `client` | 限速端：`client` / `server`    |
| `transport.proxyProtocolVersion` | 否  |          | PROXY Protocol：`v1` / `v2` |
