---
sidebar_position: 1
sidebar_label: Introduction
slug: /intro
---

# Introduction

**Orbien** is a high-performance NAT traversal tool written in **Rust**. It exposes services on a private network to the public internet securely and efficiently. By establishing a stable reverse tunnel, it works around **firewall** restrictions and is well suited to **remote debugging**, **publishing internal services**, **development and testing**, **IoT device** access, and similar scenarios.

## Features

- **Multi-protocol tunnels**: TCP, UDP, HTTP, HTTPS and SOCKS5
- **Transport**: TCP, QUIC, KCP, and WebSocket
- **Multiplexing**: TCP multiplexing so one physical connection carries multiple logical data streams, reducing overhead and latency
- **HTTPS**: HTTPS is forwarded by SNI; domain certificates stay local. Optional client-side TLS termination
- **Security**: TLS and mTLS encrypted transport
- **Operations**: A lightweight server web UI and a cross-platform desktop client for ops and monitoring
- **Cross-platform**: Linux, Windows, macOS, and FreeBSD, multiple architectures
