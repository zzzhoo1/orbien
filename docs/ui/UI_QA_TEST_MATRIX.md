# Orbien UI QA Test Matrix

本文用于验证 Sprint 1 的主题、基础组件和四个样板页。

## 1. 主题矩阵

| 页面/组件 | Light | Dark | 系统主题 | 不改变布局 | 结果 |
|---|---|---|---|---|---|
| Desktop Launch | [ ] | [ ] | [ ] | [ ] | 待填 |
| Server Login | [ ] | [ ] | [ ] | [ ] | 待填 |
| Server Monitor | [ ] | [ ] | [ ] | [ ] | 待填 |
| Server Proxies | [ ] | [ ] | [ ] | [ ] | 待填 |
| BaseButton | [ ] | [ ] | N/A | [ ] | 待填 |
| BaseField | [ ] | [ ] | N/A | [ ] | 待填 |

## 2. 键盘与焦点

- [ ] Tab 顺序符合视觉阅读顺序
- [ ] 每个可聚焦控件有清晰焦点指示器
- [ ] 焦点不被背景或边框遮挡
- [ ] Enter 可提交登录 / 触发按钮
- [ ] Space 可触发按钮
- [ ] Escape 可关闭弹层
- [ ] 焦点不因 loading 或 error 丢失
- [ ] 禁用控件不在 Tab 顺序中

## 3. BaseButton 测试

| 测试项 | Primary | Secondary | Ghost | Destructive | Icon |
|---|---|---|---|---|---|
| Default | [ ] | [ ] | [ ] | [ ] | [ ] |
| Hover | [ ] | [ ] | [ ] | [ ] | [ ] |
| Focus | [ ] | [ ] | [ ] | [ ] | [ ] |
| Disabled | [ ] | [ ] | [ ] | [ ] | [ ] |
| Loading | [ ] | [ ] | [ ] | [ ] | [ ] |
| Dark | [ ] | [ ] | [ ] | [ ] | [ ] |

- [ ] Loading 时不能重复提交
- [ ] Loading 时宽度不跳动
- [ ] Disabled 时不触发动作
- [ ] Icon-only 有 `aria-label`

## 4. 状态矩阵

| 页面 | Loading | Empty | Error | Disabled | Success/Running | Retry |
|---|---|---|---|---|---|---|
| Launch | [ ] | N/A | [ ] | [ ] | [ ] | [ ] |
| Login | [ ] | N/A | [ ] | [ ] | [ ] | [ ] |
| Monitor | [ ] | [ ] | [ ] | N/A | [ ] | [ ] |
| Proxies | [ ] | [ ] | [ ] | N/A | [ ] | [ ] |

## 5. 业务回归

### Desktop Launch
- [ ] 启动 / 停止动作调用原有 API
- [ ] 状态与真实状态一致
- [ ] 查看日志入口正常
- [ ] IPC 不可用时有提示

### Server Login
- [ ] 登录路由可访问
- [ ] 鉴权失败反馈正常
- [ ] 失败后保留输入
- [ ] 成功后跳转 Monitor
- [ ] 刷新不破坏登录守卫

### Server Monitor
- [ ] 数据来自原有 store
- [ ] 自动刷新正常
- [ ] 接口失败可重试

### Server Proxies
- [ ] 搜索 / 筛选 / 分页有效
- [ ] 搜索与筛选可组合
- [ ] 点击 / 键盘可进入 Proxy Detail

## 6. 构建结果

```
Desktop build:
Server UI typecheck:
Server UI build:
Browser console:
测试日期:
```

## 7. 通过标准

- [ ] 所有 P0 测试通过
- [ ] 无新增业务回归
- [ ] 双端 build 通过
- [ ] Light/Dark 检查通过
- [ ] 键盘与焦点检查通过
- [ ] 截图已保存
- [ ] 已知问题已记录并分级
