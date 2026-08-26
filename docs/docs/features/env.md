---
sidebar_position: 6
sidebar_label: 环境变量
title: 环境变量
---

# 环境变量

配置文件可用 `{{env.NAME}}` 引用进程环境变量，**Docker**、**systemd**、**K8s** 等凡能注入环境变量的场景都适用。

:::warning
**GUI**客户端（`Orbien-Desktop`）不支持该语法，请在界面里填写实际值，不要填写表达式！
:::

## 语法

```toml
[auth]
token = "{{env.ORBIEN_TOKEN}}"

[[tunnels]]
name = "ssh"
protocol = "tcp"
service = "127.0.0.1:22"
remotePort = {{ env.SSH_REMOTE_PORT:9000 } }
```

| 写法                                    | 含义               |
|---------------------------------------|------------------|
| <code>{'{{env.NAME}}'}</code>         | 必填。变量未设置或为空时启动失败 |
| <code>{'{{env.NAME:default}}'}</code> | 可选。未设置或为空时使用默认值  |
| <code>{'"{{env.NAME}}"'}</code>       | 字符串字段，占位符必须放在引号内 |
| <code>{'{{env.NAME}}'}</code>         | 数字、布尔字段，不要加引号    |

变量名只能包含字母、数字、下划线，且不能以数字开头。花括号内空白可有可无：<code>{'{{ env.NAME }}'}</code> 合法。

第一个 `:` 才是默认值分隔符，因此地址类默认值可以直接写：

```toml
listen = "{{env.ORBIEN_LISTEN:0.0.0.0:9527}}"
server = "{{env.ORBIEN_SERVER:127.0.0.1:9527}}"
```

密钥建议用必填写法 <code>{'{{env.ORBIEN_TOKEN}}'}</code>，不要给 token 写一个可猜测的默认值。

