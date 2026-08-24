---
sidebar_position: 2
sidebar_label: 管理界面
title: 管理界面
---

# 管理界面

服务端 Web 管理面板。在 `orbien-server.toml` 中配置 `[webServer]`，且 `port > 0` 时启用。

## 示例

```toml
# orbien-server.toml
bindAddr = "0.0.0.0"
bindPort = 9527

[webServer]
addr = "0.0.0.0"
port = 8020
user = "admin"
password = "123456"
assetsDir = "/path/to/dist"
```

浏览器访问 `http://SERVER_IP:8020`，使用 Basic Auth（`user` / `password`）登录。

`addr` 默认为 `127.0.0.1`（仅本机）。需远程访问时设为 `0.0.0.0`。`assetsDir` 可省略，省略时使用内置前端。

## 参数

| 参数                    | 必填 | 默认值         | 说明                   |
|-----------------------|----|-------------|----------------------|
| `webServer.addr`      | 否  | `127.0.0.1` | 监听地址；远程访问需 `0.0.0.0` |
| `webServer.port`      | 是  | `0`         | 监听端口；`0` 表示关闭        |
| `webServer.user`      | 否  |             | Basic Auth 用户名       |
| `webServer.password`  | 否  |             | Basic Auth 密码        |
| `webServer.assetsDir` | 否  |             | 静态资源目录；空则使用内置前端      |
| `webServer.disableAuth` | 否  | `false`     | 禁用认证（仅用于测试）      |

:::warning 安全提示
- 必须配置 `user` 和 `password` 以保护管理界面
- 如果两者都为空且未设置 `disableAuth=true`，所有请求将被拒绝
- `disableAuth=true` 仅应在测试环境中使用，生产环境必须配置认证
:::

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

