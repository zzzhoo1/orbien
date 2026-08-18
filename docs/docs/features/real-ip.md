---
sidebar_position: 5
sidebar_label: 获取真实 IP
title: 获取真实 IP
---

# 获取真实 IP

穿透后本地服务默认看到的是隧道地址。可用以下方式拿到访客真实 IP。

## PROXY Protocol

客户端连接本地服务时写入 PROXY Protocol 头。本地服务需支持解析（如 Nginx、HAProxy）。

```toml
# orbien.toml
serverAddr = "YOUR_SERVER_IP"
serverPort = 9527

[[proxies]]
name = "web"
type = "tcp"
localIP = "127.0.0.1"
localPort = 80
remotePort = 9000
transport.proxyProtocolVersion = "v2"
```

适用于 `tcp` / `udp` / `http` / `https`（透传）。`https2http` 不可用。

| 参数                               | 必填 | 默认值 | 说明                |
|----------------------------------|----|-----|-------------------|
| `transport.proxyProtocolVersion` | 否  |     | `v1` / `v2`；空表示关闭 |

## X-Forwarded-For

`http` 由服务端自动注入；`https` + `https2http` 由客户端插件自动注入，无需额外配置：

- `X-Forwarded-For`：访客 IP
- `X-Forwarded-Proto`：`http` 或 `https`

应用从请求头读取即可。

## 服务端前置负载均衡

若 orbien-server 前还有 CDN / 负载均衡，需在服务端开启 PROXY Protocol，接收上游传递的真实 IP：

```toml
# orbien-server.toml
bindAddr = "0.0.0.0"
bindPort = 9527

proxyProtocol = true
proxyProtocolTrustedCidrs = ["10.0.0.0/8", "192.168.0.0/16"]
proxyProtocolTimeoutSecs = 5
denySrcCidrs = ["1.2.3.4/32"]
```

| 参数                          | 必填 | 默认值     | 说明                        |
|-----------------------------|----|---------|---------------------------|
| `proxyProtocol`             | 否  | `false` | 是否解析上游 PROXY Protocol     |
| `proxyProtocolTrustedCidrs` | 否  |         | 信任的上游 CIDR；空则不解析 PP 头 |
| `proxyProtocolTimeoutSecs`  | 否  | `5`     | 读取 PP 头超时（秒）              |
| `denySrcCidrs`              | 否  |         | 拒绝的访客源 CIDR               |
