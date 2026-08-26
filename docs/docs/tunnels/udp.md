---
sidebar_position: 2
sidebar_label: UDP
title: UDP
---

# UDP

将内网 UDP 服务映射到服务端公网端口。

## 示例：穿透 DNS

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
name = "dns"
protocol = "udp"
service = "127.0.0.1:53"
remotePort = 9000
```

外网访问：

```shell
dig @YOUR_SERVER_IP -p 9000 example.com
```

## 参数

| 参数                               | 必填 | 默认值         | 说明                          |
|----------------------------------|----|-------------|-----------------------------|
| `udpPacketSize`                  | 否  | `1500`      | UDP 最大报文长度，客户端与服务端需一致          |
| `name`                           | 是  |             | 隧道名称，唯一                     |
| `protocol`                       | 是  |             | 固定为 `udp`                   |
| `service`                        | 是  |             | 本地服务地址，如 `127.0.0.1:53`    |
| `remotePort`                     | 是  |             | 服务端对外监听端口                   |
| `transport.bandwidth`            | 否  | `0`         | 带宽上限（Mbps）；`0` 表示不限制       |
| `transport.bandwidthLimitSide`   | 否  | `client`    | 限速端：`client` / `server`     |
| `transport.proxyProtocolVersion` | 否  |             | PROXY Protocol：`v1` / `v2`  |
