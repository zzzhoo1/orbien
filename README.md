<div align="center">
  <img src="docs/static/img/logo.png" alt="Logo" width="180" height="180" style="border-radius:24px;margin-bottom:20px;"/>
</div>
<p align="center" style="font-size:18px;color:#555;margin-top:-10px;margin-bottom:24px;">
  ![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/zzzhoo1/orbien?utm_source=oss&utm_medium=github&utm_campaign=zzzhoo1%2Forbien&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)
  Intranet penetration built with Rust and Tokio
</p>
<div align="center">
  <a href="https://github.com/orbien-org/orbien/stargazers">
    <img src="https://img.shields.io/github/stars/orbien-org/orbien?style=for-the-badge&logo=github" alt="GitHub Stars"/>
  </a>
  <a href="https://github.com/orbien-org/orbien/forks">
    <img src="https://img.shields.io/github/forks/orbien-org/orbien?style=for-the-badge&logo=github" alt="GitHub Forks"/>
  </a>
  <a href="https://github.com/orbien-org/orbien/issues">
    <img src="https://img.shields.io/github/issues/orbien-org/orbien?style=for-the-badge&logo=github" alt="GitHub Issues"/>
  </a>
  <a href="https://github.com/orbien-org/orbien/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/orbien-org/orbien?style=for-the-badge" alt="License"/>
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Rust-Tokio-orange?style=for-the-badge&logo=rust&logoColor=white" alt="Rust"/>
  </a>
  <a href="https://github.com/orbien-org/orbien/releases">
    <img src="https://img.shields.io/badge/orbien-2.1.0-blue?style=for-the-badge" alt="orbien:2.1.0"/>
  </a>
  <a href="https://somsubhra.github.io/github-release-stats/?username=orbien-org&repository=orbien">
    <img src="https://img.shields.io/github/downloads/orbien-org/orbien/total?style=for-the-badge" alt="Downloads"/>
  </a>
  <a href="https://discord.gg/4dgQjCS3k">
    <img src="https://img.shields.io/badge/Discord-Join-5865F2?style=for-the-badge&logo=discord&logoColor=white" alt="Discord"/>
  </a>
</div>

<div align="center">
  <a href="README.md"><strong>English</strong></a> &nbsp;|&nbsp;
  <a href="README_ZH.md"><strong>简体中文</strong></a>
  &nbsp;|&nbsp;
  <a href="https://orbien-org.github.io/orbien/"><strong>Docs</strong></a>
</div>

![dashboard.png](doc/img/dashboard.png)

A lightweight, high-performance, and secure intranet penetration tool with a binary size of around `5MB`.

## Features

- **High performance**: end-to-end zero-copy forwarding, low latency, high throughput, no GC pauses, and low memory usage
- **Proxy protocols**: TCP, UDP, HTTP, HTTPS, and more
- **Transport protocols**: TCP, KCP, WebSocket, QUIC, with TCP multiplexing support
- **Security**: Token-based tunnel authentication, TLS and mTLS encryption; HTTPS supports transparent forwarding and client-side TLS termination
- **Cross-platform**: Windows, Linux, macOS, FreeBSD, and more
- **Operations**: lightweight Web admin UI and cross-platform desktop client for easy configuration and monitoring

## Quick Start

[Download](download.mdx) the binary archive for your platform and extract it.

### Server

```toml
# orbien-server.toml
bindAddr = "0.0.0.0"
bindPort = 9527
```

```shell
./orbien-server -c orbien-server.toml
```

### Client

```toml
# orbien.toml
serverAddr = "127.0.0.1"
serverPort = 9527

[[proxies]]
name = "mysql"
type = "tcp"
localIP = "127.0.0.1"
localPort = 3306
remotePort = 6050
```

```shell
./orbien -c orbien.toml
```

If you prefer not to use the CLI, try the [Orbien-Desktop](https://github.com/orbien-org/orbien/releases) desktop client — built with `Tauri`, under `10MB`.

![desktop_en.gif](doc/img/desktop_en.gif)

# Benchmark

The chart below shows results from local loopback tests. Orbien keeps memory usage very low and stable.

![mem.png](doc/img/mem.png)

# Acknowledgements

This project draws architectural inspiration from [frp](https://github.com/fatedier/frp). The desktop client's UI layout is inspired by [frpc-desktop](https://github.com/luckjiawei/frpc-desktop). Thanks to the open-source community.
