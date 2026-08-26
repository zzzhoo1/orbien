---
sidebar_position: 1
sidebar_label: 身份认证
title: 身份认证
---

# 身份认证

客户端连接服务端时的 Token 鉴权。服务端未配置 `token`（或为空）时不校验。

服务端开启鉴权后，两端 `token` 必须一致，否则登录失败。Token 从配置文件读取，也可用环境变量注入，见 [环境变量](./env.md)。

:::tip
登录时用 token 计算摘要，不会把 token 明文发给服务端!
:::

## 示例

服务端：

```toml
# orbien-server.toml
[auth]
token = "YOUR_TOKEN"
```

客户端：

```toml
# orbien.toml
[auth]
token = "YOUR_TOKEN"
```

## 参数

| 参数           | 必填 | 默认值 | 说明                     |
|--------------|----|-----|------------------------|
| `auth.token` | 否  |     | 共享密钥；服务端为空表示关闭鉴权；两端需一致 |

## Token 权限策略

登录通过后，还可按 token 限制能注册的 tunnel、协议和远程端口。未给某个 token 配置策略时，该 token 不受额外限制。策略里某一项为空时，该项不限制。

```toml
# orbien-server.toml
[auth]
token = "YOUR_TOKEN"

[[auth.token_policies]]
token = "team-a"
allowed_tunnels = ["db-prod", "metrics"]
allowed_protocols = ["tcp", "udp"]
allowed_remote_ports = [3306, 9125]
```

| 参数 | 必填 | 默认值 | 说明 |
|---|---|---|---|
| `auth.token_policies[].token` | 是 | | 要限制的客户端 token |
| `auth.token_policies[].allowed_tunnels` | 否 | `[]` | 允许注册的 proxy 名称；空表示不限制 |
| `auth.token_policies[].allowed_protocols` | 否 | `[]` | 允许的协议，如 `tcp` / `udp` / `http` / `https`；空表示不限制 |
| `auth.token_policies[].allowed_remote_ports` | 否 | `[]` | 允许的远程端口；空表示不限制 |

HTTP / HTTPS 没有 `remotePort` 时，若该 token 配置了 `allowed_remote_ports`，注册会被拒绝。策略会在 dashboard 的 Monitor 页展示。

## 命令行

服务端也可通过参数设置（会覆盖配置文件中的 `token`）：

```shell
./orbien-server -c orbien-server.toml -t YOUR_TOKEN
```

| 参数               | 默认值 | 说明   |
|------------------|-----|------|
| `-t` / `--token` |     | 共享密钥 |
