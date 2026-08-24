# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 并采用 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added
- 为 `orbien-core` 与 `orbien-client` 补充单元测试（配置解析、限流、消息编解码、代理协议、X-Forwarded-For、run_id、HTTPS2HTTP 请求改写），工作区测试总数从 28 增至 81。
- 新增 Prometheus 指标端点 `GET /metrics`，以标准文本格式暴露客户端数、连接数、总流量与按代理的流量/连接数，便于直接抓取监控。
- 文档：dashboard 页新增 Prometheus 监控章节（指标列表与抓取配置）；新增《故障排查》指南。

## [0.1.0] - 2026-08-24

首个统一版本。统一了 Cargo / Tauri / package.json / Java pom 的版本号为 `0.1.0`。

### Added
- 内网穿透核心：TCP / UDP / HTTP / HTTPS 代理
- 传输协议：TCP、KCP、WebSocket、QUIC，支持 TCP 多路复用（yamux）
- 安全：Token 隧道鉴权、TLS / mTLS 加密、HTTPS 透明转发与客户端 TLS 终止
- 限流：按代理配置带宽限制（KB / MB），支持 client / server 模式
- 连接限制：按代理配置最大并发连接数 `maxConnections`
- 控制心跳：服务端控制连接心跳与超时检测
- Token 级隧道访问策略：按 token 限制协议与远程端口
- WebAuthn 登录（passkey）+ 会话 Cookie 认证，与 Basic Auth 并存
- Web 管理界面：深色仪表盘、客户端 / 代理监控、踢出代理 API
- 跨平台桌面客户端（Tauri）与 Java 客户端（Spring Boot starter）
- HTTPS2HTTP 插件：在 agent 端终结 TLS 并转发到本地 HTTP

### Changed
- 鉴权摘要算法由 MD5 升级为 HMAC-SHA256（`privilege_key`）
- 消息读取增加超时，防止任务饥饿
- 提升 `JOIN_BUF` 至 512KB，调整 yamux 接收窗口

### Security
- 升级 yamux 0.13 → 0.14.0（GHSA-4w32-2493-32g7），固定 rustls 0.23.41
- 忽略 PROXY 协议，除非配置了受信 CIDR
- 拒绝过期的鉴权时间戳，收紧代理注册

### Fixed
- 修复 PROXY v1/v2 解析与构建的边界情况
- 修复 QUIC 客户端连接端口（应连 quicBindPort 而非 bindPort）
- 修复会话 / 代理替换时的资源泄漏
- 修复 dashboard 401 时重定向到登录页

## [0.0.4] - 2026-08-23

### Added
- 服务端 Web 管理界面初步版本
- 发布脚本与 musl 兼容包

## [0.0.3] - 2026-08-23

### Added
- Orbien v2 核心代码提交
- README 与文档

[Unreleased]: https://github.com/zzzhoo1/orbien/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/zzzhoo1/orbien/compare/v0.0.4...v0.1.0
[0.0.4]: https://github.com/zzzhoo1/orbien/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/zzzhoo1/orbien/releases/tag/v0.0.3
