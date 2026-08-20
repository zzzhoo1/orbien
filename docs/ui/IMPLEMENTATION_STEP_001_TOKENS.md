# Implementation Step 001 — Theme Tokens

本文定义第一笔代码改动的具体执行内容：只建立 `desktop` 与 `server-ui` 的语义主题 tokens，不修改业务逻辑。

## 1. 目标

- 建立 Light/Dark 主题基础
- 统一背景、surface、文本、边框、强调色和状态色
- 统一圆角、间距、阴影和控件高度
- 不改变路由、API、store、Tauri IPC 和业务状态

## 2. 修改文件

```
desktop/src/styles/tokens.css
desktop/src/styles/themes.css
desktop/src/styles/main.css

server-ui/src/styles/tokens.css
server-ui/src/styles/themes.css
server-ui/src/styles/main.css
```

## 3. tokens.css

```css
:root {
  color-scheme: light;

  --color-bg: #f5f5f7;
  --color-surface: #ffffff;
  --color-surface-secondary: #f2f2f7;

  --color-text-primary: #1d1d1f;
  --color-text-secondary: #6e6e73;
  --color-text-tertiary: #8e8e93;

  --color-border: rgba(60, 60, 67, 0.18);

  --color-accent: #0071e3;
  --color-accent-hover: #0077ed;

  --color-success: #28a745;
  --color-warning: #ff9f0a;
  --color-danger: #ff3b30;

  --radius-sm: 10px;
  --radius-md: 14px;
  --radius-lg: 18px;

  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.08);
  --shadow-md: 0 8px 24px rgba(0, 0, 0, 0.1);

  --control-height-sm: 32px;
  --control-height-md: 40px;
  --control-height-lg: 48px;

  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
}
```

## 4. themes.css

```css
[data-theme="dark"] {
  color-scheme: dark;

  --color-bg: #111113;
  --color-surface: #1c1c1e;
  --color-surface-secondary: #2c2c2e;

  --color-text-primary: #f5f5f7;
  --color-text-secondary: #a1a1a6;
  --color-text-tertiary: #8e8e93;

  --color-border: rgba(84, 84, 88, 0.65);

  --color-accent: #2997ff;
  --color-accent-hover: #409cff;

  --color-success: #32d74b;
  --color-warning: #ffd60a;
  --color-danger: #ff453a;

  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 8px 24px rgba(0, 0, 0, 0.36);
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    color-scheme: dark;
    --color-bg: #111113;
    --color-surface: #1c1c1e;
    --color-surface-secondary: #2c2c2e;
    --color-text-primary: #f5f5f7;
    --color-text-secondary: #a1a1a6;
    --color-text-tertiary: #8e8e93;
    --color-border: rgba(84, 84, 88, 0.65);
    --color-accent: #2997ff;
    --color-accent-hover: #409cff;
    --color-success: #32d74b;
    --color-warning: #ffd60a;
    --color-danger: #ff453a;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.3);
    --shadow-md: 0 8px 24px rgba(0, 0, 0, 0.36);
  }
}

:focus-visible {
  outline: 3px solid color-mix(in srgb, var(--color-accent) 45%, transparent);
  outline-offset: 2px;
}

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

## 5. 入口引入

```ts
import '@/styles/tokens.css'
import '@/styles/themes.css'
import '@/styles/main.css'
```

## 6. 执行顺序

1. 记录当前构建基线
2. 创建 styles 目录
3. 创建 tokens.css
4. 创建 themes.css
5. 在入口引入
6. 将全局背景和主文本替换为 tokens
7. 运行 Desktop build
8. 运行 Server UI typecheck
9. 运行 Server UI build
10. 检查 Light/Dark 页面
11. 记录结果和截图

## 7. 验收标准

- [ ] Desktop 成功构建
- [ ] Server UI typecheck 通过
- [ ] Server UI 成功构建
- [ ] Light 页面正常
- [ ] Dark 页面正常
- [ ] `:focus-visible` 可见
- [ ] reduced-motion 已添加
- [ ] 没有修改 API / 路由 / store / Tauri IPC
- [ ] 无新增硬编码颜色

## 8. 提交信息

```
feat(ui): add semantic light and dark theme tokens
```

## 9. 下一步

本步骤通过后进入：

```
Implementation Step 002 — BaseButton
```
