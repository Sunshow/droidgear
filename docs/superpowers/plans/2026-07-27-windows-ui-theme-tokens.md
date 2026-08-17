# Windows UI Theme Tokens Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将全局主题 token 调整为 Windows 11 气质（Win11 蓝强调、表面分层、更大圆角），深浅双主题可用，默认深色优先，不改页面结构。

**Architecture:** 仅修改 shadcn 语义 CSS 变量与默认 theme 值；组件通过现有 token 自动继承。不引入新组件、不改布局 JSX。

**Tech Stack:** Tailwind v4, shadcn/ui tokens, OKLCH, React ThemeProvider, Tauri preferences 默认值

## Global Constraints

- 不改页面结构 / 侧栏宽度 / 业务组件 JSX
- 不实现 Mica/Acrylic / 系统强调色 API
- 不迁移已有用户主题偏好
- 改完后 `npm run check:all` 必须通过
- 默认字号：`15px`，行高：`1.5`，圆角：`0.75rem`

## File Map

| File                                 | Responsibility                                        |
| ------------------------------------ | ----------------------------------------------------- |
| `src/theme-variables.css`            | light/dark 语义色 + radius                            |
| `src/App.css`                        | 启动底色 + body 字号行高                              |
| `src/components/ThemeProvider.tsx`   | `defaultTheme = 'dark'`                               |
| `src/services/preferences.ts`        | 前端 fallback 默认 `theme: 'dark'`                    |
| `src-tauri/src/utils/preferences.rs` | 后端默认 preferences 若存在 theme 默认值则同步为 dark |

---

### Task 1: 更新 theme tokens

**Files:**

- Modify: `src/theme-variables.css`
- Modify: `src/App.css`

- [ ] **Step 1: 替换 `:root` / `.dark` 色值与 radius**

在 `src/theme-variables.css` 中保留 `@theme inline` 不变，将 `:root` 与 `.dark` 替换为：

```css
:root {
  --background: oklch(0.985 0.004 255);
  --foreground: oklch(0.22 0.02 255);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.22 0.02 255);
  --popover: oklch(1 0 0);
  --popover-foreground: oklch(0.22 0.02 255);
  --primary: oklch(0.48 0.16 255);
  --primary-foreground: oklch(0.99 0.01 255);
  --secondary: oklch(0.955 0.01 255);
  --secondary-foreground: oklch(0.28 0.03 255);
  --muted: oklch(0.955 0.01 255);
  --muted-foreground: oklch(0.48 0.02 255);
  --accent: oklch(0.94 0.02 255);
  --accent-foreground: oklch(0.28 0.03 255);
  --destructive: oklch(0.58 0.22 27);
  --border: oklch(0.9 0.01 255);
  --input: oklch(0.9 0.01 255);
  --ring: oklch(0.55 0.14 255);
  --chart-1: oklch(0.72 0.12 250);
  --chart-2: oklch(0.62 0.16 255);
  --chart-3: oklch(0.52 0.18 258);
  --chart-4: oklch(0.45 0.16 262);
  --chart-5: oklch(0.4 0.14 266);
  --radius: 0.75rem;
  --sidebar: oklch(0.975 0.006 255);
  --sidebar-foreground: oklch(0.22 0.02 255);
  --sidebar-primary: oklch(0.48 0.16 255);
  --sidebar-primary-foreground: oklch(0.99 0.01 255);
  --sidebar-accent: oklch(0.94 0.02 255);
  --sidebar-accent-foreground: oklch(0.28 0.03 255);
  --sidebar-border: oklch(0.9 0.01 255);
  --sidebar-ring: oklch(0.55 0.14 255);
}

.dark {
  --background: oklch(0.17 0.012 255);
  --foreground: oklch(0.96 0.01 255);
  --card: oklch(0.22 0.014 255);
  --card-foreground: oklch(0.96 0.01 255);
  --popover: oklch(0.22 0.014 255);
  --popover-foreground: oklch(0.96 0.01 255);
  --primary: oklch(0.68 0.14 255);
  --primary-foreground: oklch(0.16 0.02 255);
  --secondary: oklch(0.27 0.016 255);
  --secondary-foreground: oklch(0.96 0.01 255);
  --muted: oklch(0.27 0.016 255);
  --muted-foreground: oklch(0.76 0.02 255);
  --accent: oklch(0.32 0.03 255);
  --accent-foreground: oklch(0.96 0.01 255);
  --destructive: oklch(0.704 0.191 22.216);
  --border: oklch(1 0 0 / 10%);
  --input: oklch(1 0 0 / 12%);
  --ring: oklch(0.68 0.14 255);
  --chart-1: oklch(0.72 0.12 250);
  --chart-2: oklch(0.62 0.16 255);
  --chart-3: oklch(0.52 0.18 258);
  --chart-4: oklch(0.45 0.16 262);
  --chart-5: oklch(0.4 0.14 266);
  --sidebar: oklch(0.2 0.014 255);
  --sidebar-foreground: oklch(0.96 0.01 255);
  --sidebar-primary: oklch(0.68 0.14 255);
  --sidebar-primary-foreground: oklch(0.16 0.02 255);
  --sidebar-accent: oklch(0.28 0.02 255);
  --sidebar-accent-foreground: oklch(0.96 0.01 255);
  --sidebar-border: oklch(1 0 0 / 10%);
  --sidebar-ring: oklch(0.68 0.14 255);
}
```

- [ ] **Step 2: 更新 `App.css` 启动底色与字号**

```css
html {
  background-color: oklch(0.17 0.012 255);
}

#root {
  background-color: oklch(0.17 0.012 255);
  min-height: 100vh;
}
```

`body` 内：

```css
font-size: 15px;
line-height: 1.5;
```

- [ ] **Step 3: 目视确认 CSS 语法无误（无未闭合括号）**

---

### Task 2: 默认主题改为 dark

**Files:**

- Modify: `src/components/ThemeProvider.tsx`
- Modify: `src/services/preferences.ts`
- Modify: `src-tauri/src/utils/preferences.rs`（若有默认 theme）

- [ ] **Step 1: ThemeProvider**

```tsx
defaultTheme = 'dark',
```

- [ ] **Step 2: preferences.ts fallback**

```ts
return {
  theme: 'dark',
  language: null,
  terminal_font_family: null,
  terminal_shell_command: null,
}
```

- [ ] **Step 3: 同步 Rust 默认值**

在 `src-tauri/src/utils/preferences.rs` 中将默认 `theme` 从 `"system"` 改为 `"dark"`（若存在该默认）。

---

### Task 3: 验证与收尾

**Files:** none new

- [ ] **Step 1: 运行检查**

```bash
npm run check:all
```

Expected: 零错误。若测试硬编码 `theme: 'system'` 且断言默认值，按需改为 `dark`。

- [ ] **Step 2: 提交**

```bash
git add src/theme-variables.css src/App.css src/components/ThemeProvider.tsx src/services/preferences.ts src-tauri/src/utils/preferences.rs docs/superpowers/plans/2026-07-27-windows-ui-theme-tokens.md
git commit -m "feat(ui): Windows 11 style theme tokens and dark default"
```

---

## Spec Coverage Checklist

- [x] Win11 蓝 primary / ring
- [x] dark surface 分层
- [x] light 同色相
- [x] radius 0.75rem
- [x] body 15px / 1.5
- [x] 启动底色对齐 dark
- [x] defaultTheme dark
- [x] preferences 默认 dark
- [x] 不改布局
