---
sidebar_position: 6
sidebar_label: Environment Variables
title: Environment Variables
---

# Environment Variables

Config files can reference process environment variables with `{{env.NAME}}`. This works anywhere you can inject env vars, including **Docker**, **systemd**, and **Kubernetes**.

:::warning
The GUI client (`Orbien-Desktop`) does not support this syntax. Enter the actual value in the UI, not an expression.
:::

## Syntax

```toml
[auth]
token = "{{env.ORBIEN_TOKEN}}"

[[tunnels]]
name = "ssh"
protocol = "tcp"
service = "127.0.0.1:22"
remotePort = {{ env.SSH_REMOTE_PORT:9000 } }
```

| Form                                    | Meaning                                                                 |
|-----------------------------------------|-------------------------------------------------------------------------|
| <code>{'{{env.NAME}}'}</code>           | Required. Startup fails if the variable is unset or empty               |
| <code>{'{{env.NAME:default}}'}</code>   | Optional. Uses the default if the variable is unset or empty            |
| <code>{'"{{env.NAME}}"'}</code>         | String fields: the placeholder must be inside quotes                    |
| <code>{'{{env.NAME}}'}</code>           | Numeric and boolean fields: do not quote                                |

Variable names may contain only letters, digits, and underscores, and must not start with a digit. Whitespace inside the braces is optional: <code>{'{{ env.NAME }}'}</code> is valid.

Only the first `:` is the default-value separator, so address-style defaults can be written as-is:

```toml
listen = "{{env.ORBIEN_LISTEN:0.0.0.0:9527}}"
server = "{{env.ORBIEN_SERVER:127.0.0.1:9527}}"
```

For secrets, prefer the required form <code>{'{{env.ORBIEN_TOKEN}}'}</code>. Do not give the token a guessable default.
