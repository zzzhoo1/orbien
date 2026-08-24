---
sidebar_position: 7
sidebar_label: 故障排查
title: 故障排查
---

# 故障排查

本文汇总 Orbien 使用中常见问题的排查思路。排查前请先确认版本与配置：

```shell
./orbien-server --version
./orbien --version
```

## 目录

- [客户端连不上服务端](#客户端连不上服务端)
- [隧道通了但访问不通](#隧道通了但访问不通)
- [HTTP / HTTPS 代理异常](#http--https-代理异常)
- [UDP 代理异常](#udp-代理异常)
- [登录鉴权失败](#登录鉴权失败)
- [性能与连接数问题](#性能与连接数问题)
- [如何收集诊断信息](#如何收集诊断信息)

## 客户端连不上服务端

**现象**：客户端日志反复出现 `failed to establish control, retry in 3s` 或 `Connection refused`。

**排查步骤**：

1. 确认服务端进程在运行，且监听端口正确：
   ```shell
   ss -tlnp | grep 9527
   ```
2. 确认客户端配置的 `serverAddr` / `serverPort` 与服务端 `bindAddr` / `bindPort` 一致。
3. 若中间有防火墙 / 安全组，放行对应端口（TCP 控制端口，以及启用 QUIC/KCP 时的 UDP 端口）。
4. 若使用域名，确认 `serverAddr` 能解析到服务端公网 IP。
5. 检查服务端日志是否有 `client logged in`；若没有，说明连接未到达服务端。

## 隧道通了但访问不通

**现象**：客户端已登录（服务端显示 online），但访问 `remotePort` 不通。

**排查步骤**：

1. 确认代理已注册：服务端日志应有 `tcp proxy listening ... proxy=<name>`。
2. 确认客户端 `localIP` / `localPort` 指向的服务确实在监听：
   ```shell
   ss -tlnp | grep <localPort>
   ```
3. 用 `curl` 或 `nc` 直接测试本地服务，排除本地服务本身的问题。
4. 检查 `remotePort` 是否已被占用或与其它代理冲突。
5. 若启用了 TCP 多路复用（`tcpMux`），可临时关闭对比，排除 yamux 问题。

## HTTP / HTTPS 代理异常

**常见原因**：

- **域名未解析**：确认访问的域名已解析到服务端，且 `custom_domains` / `subdomain` 配置正确。
- **Host 头不匹配**：服务端按 Host 路由到对应代理。确认请求的 Host 与配置一致；需要改写时使用 `host_header_rewrite`。
- **HTTPS 证书**：若使用客户端 TLS 终止，确认证书 / 私钥路径正确；透明转发模式下无需配置证书。

**查看路由是否注册**：在服务端 dashboard 的 Proxies 页查看代理状态是否为 online。

## UDP 代理异常

**排查步骤**：

1. 确认代理类型为 `udp`，且 `localPort` / `remotePort` 配置正确。
2. 大包场景下，若丢包或截断，调大 `udpPacketSize`（默认 1500），例如：
   ```toml
   udpPacketSize = 8192
   ```
3. UDP 无连接概念，确认客户端 / 服务端两侧 UDP 端口均未被防火墙拦截。

## 登录鉴权失败

**现象**：客户端登录被拒（`LoginResp.error` 非空），或 dashboard 登录失败。

**排查步骤**：

1. **Token 鉴权**：确认客户端 `auth.token` 与服务端 `[auth]` 配置一致。服务端未配置 token 时客户端无需提供。
2. **时间同步**：`privilege_key` 基于 HMAC-SHA256(token, timestamp)，默认允许 15 分钟时钟偏移。客户端 / 服务端时间偏差过大时登录失败，需同步时钟（NTP）。
3. **Token 策略**：若配置了 `[[auth.token_policies]]`，确认该 token 被允许访问目标协议 / 端口。
4. **dashboard Basic Auth**：确认 `webServer.user` / `webServer.password` 正确；WebAuthn 用户需先完成 passkey 注册。

## 性能与连接数问题

- **连接数受限**：若代理设置了 `maxConnections`，超过上限的新连接会被拒绝。确认该值符合预期。
- **带宽受限**：若配置了 `bandwidthLimit`，流量会被限速。确认单位（KB / MB）与模式（client / server）正确。
- **吞吐偏低**：检查是否启用了不必要的加密 / 多路复用开销；确认服务端 CPU / 带宽资源充足。

## 如何收集诊断信息

排查时请收集以下信息，便于定位问题：

```shell
# 服务端日志
./orbien-server -c orbien-server.toml 2>&1 | tee /tmp/orbien-server.log

# 客户端日志（开启更详细的日志）
RUST_LOG=debug ./orbien -c orbien.toml 2>&1 | tee /tmp/orbien-client.log

# 端口监听情况
ss -tlnp

# 版本信息
./orbien-server --version
./orbien --version
```

将以上输出连同你的配置文件（注意隐去 token / 密码）一并提供。
