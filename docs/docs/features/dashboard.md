---
sidebar_position: 2
sidebar_label: 管理界面
title: 管理界面
---

# 管理界面

服务端 Web 管理面板。 `port > 0` 时启用，启用后 **必须** 配置 `user` 与
`password`。

![dashboard.png](/img/dashboard.png)

## 示例

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"

[dashboard]
addr = "0.0.0.0"
port = 8020
user = "admin"
password = "123456"
```

浏览器访问 `http://SERVER_IP:8020`，在弹出框输入用户名和密码登录。

:::tip
`addr` 默认为 `127.0.0.1`（仅本机），需远程访问时设为 `0.0.0.0`
:::

## 参数

| 参数                    | 必填 | 默认值         | 说明                   |
|-----------------------|----|-------------|----------------------|
| `webServer.addr`      | 否  | `127.0.0.1` | 监听地址；远程访问需 `0.0.0.0` |
| `webServer.port`      | 是  | `0`         | 监听端口；`0` 表示关闭        |
| `webServer.user`      | 否  |             | Basic Auth 用户名       |
| `webServer.password`  | 否  |             | Basic Auth 密码        |
| `webServer.assetsDir` | 否  |             | 静态资源目录；空则使用内置前端      |

## 命令行

无配置文件时也可用参数开启：

```shell
./orbien-server --dashboard_port 8020 --dashboard_user admin --dashboard_pwd 123456
```

| 参数                 | 默认值       | 说明          |
|--------------------|-----------|-------------|
| `--dashboard_addr` | `0.0.0.0` | 监听地址        |
| `--dashboard_port` | `0`       | 监听端口；`0` 关闭 |
| `--dashboard_user` | `admin`   | 用户名         |
| `--dashboard_pwd`  | `admin`   | 密码          |

## Token 策略

Monitor 页会展示每个 token 的活跃控制连接数，以及 `allowed_tunnels`、`allowed_protocols`、`allowed_remote_ports`。配置了策略但当前没有连接的 token 也会列出。

## Prometheus 监控

服务端内置 Prometheus 指标端点 `GET /metrics`，以标准文本格式暴露监控指标，可直接被 Prometheus / Grafana 抓取。

```shell
curl http://SERVER_IP:8020/metrics
```

返回示例：

```text
# HELP orbien_clients_online Current number of online clients.
# TYPE orbien_clients_online gauge
orbien_clients_online 3
# HELP orbien_traffic_in_bytes_total Total bytes received.
# TYPE orbien_traffic_in_bytes_total counter
orbien_traffic_in_bytes_total 12345678
# HELP orbien_proxy_connections_current Current connections per proxy.
# TYPE orbien_proxy_connections_current gauge
orbien_proxy_connections_current{proxy="web",type="http"} 5
```

### 指标列表

| 指标 | 类型 | 说明 |
|------|------|------|
| `orbien_clients_online` | gauge | 当前在线客户端数 |
| `orbien_clients_total` | gauge | 累计见过的客户端数 |
| `orbien_connections_current` | gauge | 当前活跃连接数 |
| `orbien_traffic_in_bytes_total` | counter | 累计接收字节数 |
| `orbien_traffic_out_bytes_total` | counter | 累计发送字节数 |
| `orbien_proxy_connections_current{proxy,type}` | gauge | 按代理的当前连接数 |
| `orbien_proxy_traffic_in_bytes_total{proxy}` | counter | 按代理的累计接收字节数 |
| `orbien_proxy_traffic_out_bytes_total{proxy}` | counter | 按代理的累计发送字节数 |

### Prometheus 抓取配置

```yaml
scrape_configs:
  - job_name: "orbien"
    static_configs:
      - targets: ["SERVER_IP:8020"]
    metrics_path: /metrics
```

> 注意：`/metrics` 与 dashboard 共用同一 Web 服务端口。若配置了 Basic Auth，抓取时需提供凭据（可通过 `basic_auth` 字段或反向代理注入）。

