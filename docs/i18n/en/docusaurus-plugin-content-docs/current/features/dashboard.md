---
sidebar_position: 2
sidebar_label: Dashboard
title: Dashboard
---

# Dashboard

Server web dashboard. Enabled when `port > 0`; **`user` and `password` are required** once enabled.

![dashboard_en.png](/img/dashboard_en.png)

## Example

```toml
# orbien-server.toml
listen = "0.0.0.0:9527"

[dashboard]
addr = "0.0.0.0"
port = 8020
user = "admin"
password = "123456"
```

Open `http://SERVER_IP:8020` in a browser and sign in with username and password in the login dialog.

:::tip
`addr` defaults to `127.0.0.1` (localhost only). Set it to `0.0.0.0` for remote access.
:::

## Parameters

| Parameter               | Required | Default       | Description                                  |
|-------------------------|----------|---------------|----------------------------------------------|
| `dashboard.addr`        | No       | `127.0.0.1`   | Listen address; use `0.0.0.0` for remote access |
| `dashboard.port`        | Yes      | `0`           | Listen port; `0` disables the dashboard      |
| `dashboard.user`        | Yes      |               | Login username                               |
| `dashboard.password`    | Yes      |               | Login password                               |
| `dashboard.staticDir`   | No       |               | Static assets directory; empty uses the built-in frontend |
