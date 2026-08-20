# Orbien UI Implementation Handoff

本文标志着 Orbien UI 重构从设计文档阶段进入代码实施阶段，目标是将已确认的 tokens、基础组件和 Sprint 1 故事板转化为可提交、可构建、可验收的前端改动。

## 1. 实施边界

本阶段只修改 `desktop` 和 `server-ui` 的前端表现层、组件层与主题层，不修改 Rust 核心、代理协议、API 契约、配置文件格式和 Docker 运行逻辑。

**允许修改：**
- 全局 CSS
- 主题 tokens
- Vue 组件
- 页面布局和视觉样式
- Loading、Error、Empty、Disabled 等状态展示
- 可访问性属性和键盘交互

**禁止修改：**
- API 请求地址和数据结构
- 路由路径
- Pinia store 的业务逻辑
- Tauri IPC 调用
- 代理协议
- 配置文件格式
- Rust 核心行为

## 2. 实施分支

```bash
git checkout -b feat/ui-design-system-sprint1
```

## 3. 目录规划

### Desktop
```
desktop/src/
├── components/base/
│   ├── BaseButton.vue
│   ├── BaseField.vue
│   ├── BaseCard.vue
│   ├── StatusBadge.vue
│   ├── InlineAlert.vue
│   └── StateBlock.vue
└── styles/
    ├── tokens.css
    ├── themes.css
    └── main.css
```

### Server UI
```
server-ui/src/
├── components/base/
│   ├── BaseButton.vue
│   ├── BaseField.vue
│   ├── BaseCard.vue
│   ├── StatusBadge.vue
│   ├── InlineAlert.vue
│   └── StateBlock.vue
└── styles/
    ├── tokens.css
    ├── themes.css
    └── main.css
```

## 4. BaseButton 最小接口

```ts
type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'destructive' | 'icon'
type ButtonSize = 'sm' | 'md' | 'lg'

interface ButtonProps {
  variant?: ButtonVariant
  size?: ButtonSize
  disabled?: boolean
  loading?: boolean
  block?: boolean
}
```

## 5. BaseField 最小接口

```ts
interface FieldProps {
  label?: string
  modelValue?: string | number
  placeholder?: string
  help?: string
  error?: string
  disabled?: boolean
  readonly?: boolean
  type?: 'text' | 'password' | 'number' | 'search'
}
```

## 6. 构建检查

```bash
# Desktop
cd desktop && npm ci && npm run build

# Server UI
cd server-ui && npm ci && npm run typecheck && npm run build
```

## 7. 完成标准

- tokens 已接入两个 UI
- 基础组件已实现并被页面实际使用
- 四个样板页完成基础迁移（Launch、Login、Monitor、Proxies）
- API、路由、状态管理和代理行为没有回归
- 双端构建通过
- Server UI typecheck 通过
- Light/Dark 主题检查通过
- PR 中附有截图、状态说明和测试结果
