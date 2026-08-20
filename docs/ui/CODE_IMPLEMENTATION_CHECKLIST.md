# Orbien Code Implementation Checklist

本文是从设计文档进入代码修改前的执行清单。

## Phase 0：准备

- [ ] 创建 `feat/ui-design-system-sprint1` 分支
- [ ] 记录 desktop build 基线
- [ ] 记录 server-ui typecheck 基线
- [ ] 记录 server-ui build 基线
- [ ] 截取四个样板页当前截图

```bash
git checkout -b feat/ui-design-system-sprint1
cd desktop && npm ci && npm run build
cd ../server-ui && npm ci && npm run typecheck && npm run build
```

## Phase 1：主题 tokens

**文件：**
```
desktop/src/styles/tokens.css
desktop/src/styles/themes.css
server-ui/src/styles/tokens.css
server-ui/src/styles/themes.css
```

- [ ] 背景语义变量
- [ ] surface 语义变量
- [ ] 文本变量（primary / secondary / tertiary）
- [ ] 边框变量
- [ ] accent、success、warning、danger 变量
- [ ] 间距变量
- [ ] 圆角变量
- [ ] 阴影变量
- [ ] 控件高度变量
- [ ] Light 主题
- [ ] Dark 主题
- [ ] `:focus-visible` 全局样式
- [ ] `prefers-reduced-motion` 处理
- [ ] 入口引入

**验收：**
- [ ] 主题切换不改变布局
- [ ] 文本可读
- [ ] 卡片和边框可见
- [ ] 焦点环清晰

## Phase 2：BaseButton

- [ ] primary / secondary / ghost / destructive / icon
- [ ] sm / md / lg
- [ ] hover / active / focus-visible / disabled / loading
- [ ] loading 时不能重复提交
- [ ] loading 时宽度稳定
- [ ] icon-only 的 `aria-label`
- [ ] 键盘 Tab / Enter / Space

## Phase 3：BaseField 与 SearchField

- [ ] label / helper / error
- [ ] text / password / number / search
- [ ] disabled / readonly
- [ ] label 与 input 正确关联
- [ ] error 与 input 正确关联
- [ ] SearchField 支持清空

## Phase 4：StatusBadge / BaseCard / InlineAlert / StateBlock

- [ ] StatusBadge: running / stopped / pending / warning / error / info
- [ ] BaseCard: title / subtitle / extra / footer
- [ ] InlineAlert: info / success / warning / error
- [ ] StateBlock: empty / loading / error / retrying

## Phase 5：迁移样板页

### Desktop Launch
- [ ] BaseCard / StatusBadge / BaseButton / InlineAlert
- [ ] 保留原有启停逻辑和 IPC
- [ ] 验证所有状态

### Server Login
- [ ] BaseCard / BaseField / BaseButton / InlineAlert
- [ ] 保留登录 API 和路由守卫

### Server Monitor
- [ ] BaseCard / Metric Card / StatusBadge / StateBlock
- [ ] 保留 dashboard store 和刷新逻辑

### Server Proxies
- [ ] Segmented Control / SearchField / 统一表格 / 统一分页
- [ ] 区分无代理和筛选无结果

## Phase 6：质量检查

```bash
cd desktop && npm run build
cd ../server-ui && npm run typecheck && npm run build
```

- [ ] Light / Dark 主题
- [ ] 1280px / 1024px / 768px / 390px
- [ ] 键盘 Tab / Enter / Space / Escape
- [ ] loading / empty / error / disabled
- [ ] 无新增硬编码颜色
- [ ] 无焦点丢失
- [ ] 浏览器控制台无新增错误
