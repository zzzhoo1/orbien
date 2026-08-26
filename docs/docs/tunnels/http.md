---
sidebar_position: 3
sidebar_label: HTTP
title: HTTP
---

# HTTP

通过域名将内网 HTTP 服务暴露到公网。服务端需开启域名入口，见 [域名](./domains.md)。

`domains` 必填且非空：可写完整域名（如 `web.example.com`），或无 `.` 的短前缀（如 `web`，需服务端配置 `rootDomain`），不支持通配符。

## 示例：穿透 Web 服务

服务端：

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"
httpGwPort = 80
```

客户端：

```toml
# orbien.toml
server = "YOUR_SERVER_IP:9527"

[[tunnels]]
name = "web"
protocol = "http"
service = "127.0.0.1:8080"
domains = ["web.example.com"]
```

将 `web.example.com` 解析到服务端 IP 后访问：

```shell
curl http://web.example.com
```

## 示例：Basic 鉴权

```toml
[[tunnels]]
name = "web-auth"
protocol = "http"
service = "127.0.0.1:8080"
domains = ["web.example.com"]
basicAuthUser = "alice"
basicAuthPassword = "secret"
```

访问时需带 Basic 凭证，失败返回 `401`：

```shell
curl -u alice:secret http://web.example.com
```

## 示例：按 HTTP 用户分流

同域名下可为不同 Basic 用户挂到不同本地服务；未匹配到专用路由时，会落到未设置 `routeByHTTPUser` 的隧道（若有）。

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

## 示例：改写 Host

转发到本地服务时，把请求的 `Host` 改成指定值：

```toml
[[tunnels]]
name = "web"
protocol = "http"
service = "127.0.0.1:8080"
domains = ["web.example.com"]
hostHeaderRewrite = "127.0.0.1"
```

## 参数

| 参数                               | 必填  | 默认值         | 说明                                                    |
|----------------------------------|-----|-------------|-------------------------------------------------------|
| `name`                           | 是   |             | 隧道名称，唯一                                               |
| `protocol`                       | 是   |             | 固定为 `http`                                            |
| `service`                        | 是   |             | 本地服务地址，如 `127.0.0.1:8080`                            |
| `domains`                        | 是   |             | 域名列表，至少一个；完整域名或无 `.` 的前缀（前缀需服务端 `rootDomain`） |
| `locations`                      | 否   |             | 路径前缀，如 `/api`；空表示全部                                   |
| `basicAuthUser`                  | 否   |             | HTTP Basic 鉴权用户名；与 `basicAuthPassword` 都为空则不鉴权       |
| `basicAuthPassword`              | 否   |             | HTTP Basic 鉴权密码                                               |
| `routeByHTTPUser`                | 否   |             | 按请求 Basic 用户名选路由；空表示匹配任意用户（精确用户优先）         |
| `hostHeaderRewrite`              | 否   |             | 改写转发到本地服务的 Host；空表示不改                                 |
| `transport.bandwidth`            | 否   | `0`         | 带宽上限（Mbps）；`0` 表示不限制                                 |
| `transport.bandwidthLimitSide`   | 否   | `client`    | 限速端：`client` / `server`                               |
| `transport.proxyProtocolVersion` | 否   |             | PROXY Protocol：`v1` / `v2`                            |
