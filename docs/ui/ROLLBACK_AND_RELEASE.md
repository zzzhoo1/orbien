# Orbien UI Rollback and Release

本文定义 Sprint 1 UI 改造的回滚、发布前检查和风险处理方式。

## 1. 发布前门槛

- [ ] Desktop build 通过
- [ ] Server UI typecheck 通过
- [ ] Server UI build 通过
- [ ] QA Matrix P0 项全部通过
- [ ] Light / Dark 截图已审查
- [ ] 键盘焦点已检查
- [ ] 现有 API / 路由 / store / IPC 没有回归
- [ ] PR 描述已填写
- [ ] 已知问题已记录

## 2. 风险等级

| 等级 | 示例 | 处理 |
|---|---|---|
| Blocker | 无法构建、登录失败、代理控制失效 | 禁止合并，立即回滚 |
| High | 文本不可读、主按钮不可用 | 修复后再合并 |
| Medium | 窄屏布局问题、部分状态异常 | 记录并安排修复 |
| Low | 阴影、间距、动画细节 | 后续优化 |

## 3. 回滚原则

- 优先回滚最近一个独立提交
- 不要为了回滚视觉问题修改业务逻辑
- 回滚后必须重新运行构建和 QA
- 回滚操作必须记录在 CODE_CHANGELOG_TEMPLATE.md

```bash
git log --oneline -10
git revert <commit-hash>
```

## 4. 发布顺序

1. 合并 semantic theme tokens
2. 合并 BaseButton
3. 合并 BaseField
4. 发布 Desktop Launch → 验证
5. 发布 Server Login → 验证
6. 发布 Server Monitor → 验证
7. 发布 Server Proxies → 验证
8. 完整回归

## 5. 提交规则

```
feat(ui): add semantic light and dark theme tokens
feat(ui): add base button component
feat(ui): migrate desktop launch page
fix(ui): restore visible focus ring
fix(ui): prevent repeated loading submission
```

## 6. 发布后观察

- [ ] Desktop 启动 / 停止正常
- [ ] Server 登录正常
- [ ] Monitor 数据和刷新正常
- [ ] Proxies 搜索 / 筛选 / 分页正常
- [ ] 控制台无新增严重错误
- [ ] Light / Dark 无明显异常
