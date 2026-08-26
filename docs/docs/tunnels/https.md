---
sidebar_position: 4
sidebar_label: HTTPS
title: HTTPS
---

# HTTPS

通过域名将内网服务暴露到公网。服务端需开启域名入口，见 [域名](./domains.md)。

`domains` 必填且非空：可写完整域名（如 `web.example.com`），或无 `.` 的短前缀（如 `web`，需服务端 `rootDomain`）。不支持通配符。

两种模式：

- **透传**：按 SNI 转发，证书在内网 HTTPS 服务上
- **TLS 终止**：客户端插件 `tls-term` 终止 TLS，再转发到本地 HTTP服务

## 示例：透传

服务端：

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"
httpsGwPort = 443
```

客户端：

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9527"

[[tunnels]]
name = "web-ssl"
protocol = "https"
service = "127.0.0.1:443"
domains = ["web.example.com"]
```

将 `web.example.com` 解析到服务端 IP 后访问：

```shell
curl https://web.example.com
```

## 示例：TLS 终止

客户端终止 TLS，后端只需提供 HTTP：

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

`certFile` / `keyFile` 可省略，省略时使用临时自签证书（浏览器会提示不受信任）。启用插件后使用 `plugin.service` 指向本地 HTTP，不再使用隧道级 `service`，且不可配置 PROXY Protocol。

## 示例：TLS 终止时改写 Host

在 `tls-term` 下，转发到本地 HTTP 前改写 `Host`：

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

## 示例：TLS 终止时追加请求头

在 `tls-term` 下，向本地 HTTP 追加自定义请求头：

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

## 参数

| 参数                               | 必填 | 默认值         | 说明                                                    |
|----------------------------------|----|-------------|-------------------------------------------------------|
| `name`                           | 是  |             | 隧道名称，唯一                                               |
| `protocol`                       | 是  |             | 固定为 `https`                                           |
| `service`                        | 条件 |             | 本地服务地址（透传必填），如 `127.0.0.1:443`                       |
| `domains`                        | 是  |             | 域名列表，至少一个；完整域名或无 `.` 的前缀（前缀需服务端 `rootDomain`） |
| `plugin.type`                    | 否  |             | `tls-term`：客户端终止 TLS                                |
| `plugin.service`                 | 条件 |             | `tls-term` 时必填，如 `127.0.0.1:80`                       |
| `plugin.certFile`                 | 否  |             | 证书路径；空则临时自签                                           |
| `plugin.keyFile`                 | 否  |             | 私钥路径；空则临时自签                                           |
| `plugin.hostHeaderRewrite`       | 否  |             | 改写转发到本地服务的 Host；空表示不改                                 |
| `plugin.requestHeaders.set`      | 否  |             | 向后端追加请求头，键值对                                          |
| `transport.bandwidth`            | 否  | `0`         | 带宽上限（Mbps）；`0` 表示不限制                                 |
| `transport.bandwidthLimitSide`   | 否  | `client`    | 限速端：`client` / `server`                               |
| `transport.proxyProtocolVersion` | 否  |             | PROXY Protocol：`v1` / `v2`（`tls-term` 不可用）            |
