---
sidebar_position: 3
title: Quick Start
---

# Quick Start

[Download](download.mdx) the archive for your platform and extract it.

## Server

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"
```

```shell
./orbien-server -c orbien-server.toml
```

## Client

If you find the CLI cumbersome, you can use [Orbien Desktop](download.mdx).

```toml
# orbien.toml
server = "127.0.0.1:9527"

[[tunnels]]
name = "mysql"
protocol = "tcp"
service = "127.0.0.1:3306"
remotePort = 9000
```

```shell
./orbien -c orbien.toml
```
