# Windows UI 主题 Token 焕新设计

日期：2026-07-27  
状态：已批准（待实现）  
范围：全局视觉 token，不改页面结构

## 1. 背景与目标

DroidGear 桌面端（Tauri v2 + React 19 + shadcn/ui + Tailwind v4）当前主题接近 shadcn 默认中性灰黑，Windows 下缺乏 Win11 常见的表面层级与系统蓝强调。

目标：在**不改布局与业务组件结构**的前提下，通过全局主题 token 把视觉气质调成 **Windows 11 风**，并 **深浅双主题都做、默认深色优先**。

### 成功标准

1. 深色模式具有明显表面分层（background < card/sidebar），边框更轻，主色为 Win11 蓝。
2. 浅色模式与深色同色相体系，不出现两套风格。
3. 现有 shadcn 组件无需改 JSX 即可继承新 token。
4. 正文 / muted / border 对比度可读；Windows 启动底色与 dark 一致，减少白闪。
5. `npm run check:all` 通过。

## 2. 非目标

- 不改侧栏宽度、页面布局、信息架构。
- 不做真正的 Mica / Acrylic 系统材质或 `backdrop-blur` 壳层改造（后续可选方案 B）。
- 不读取 Windows 系统强调色 API（后续可选方案 C）。
- 不改 TUI。
- 不批量清理业务代码中的硬编码颜色（仅 token；发现明显不协调再定点修）。

## 3. 方案选择

已评估三种方案：

| 方案                  | 内容                    | 结论     |
| --------------------- | ----------------------- | -------- |
| A. Token 轻量焕新     | 只改 CSS 变量与默认主题 | **采用** |
| B. Token + 亚克力表面 | 额外改壳层 class        | 暂不做   |
| C. 系统强调色联动     | 平台 API                | 暂不做   |

## 4. 设计细节

### 4.1 色板原则

使用现有 OKLCH + shadcn 语义 token 体系（`--background`、`--card`、`--primary` 等），保持 `@theme inline` 映射不变，只替换 `:root` 与 `.dark` 的值。

#### 深色（优先视觉）

- **background**：冷中性深灰，约 `oklch(0.16–0.18 …)`，避免纯黑。
- **card / popover / sidebar**：比 background 略亮一层，形成 Win11 表面层级。
- **border / input**：低对比半透明白边（约 8%–12% alpha）。
- **primary**：固定 Win11 蓝，色相约 250–255，中等 chroma；用于按钮与焦点环。
- **primary-foreground**：高对比浅色文字。
- **muted-foreground**：略提高亮度，避免过灰难读。
- **ring**：与 primary 同系，保证焦点可见。
- **sidebar-\***：与 surface 层级一致，`sidebar-primary` 对齐 primary 蓝。

#### 浅色

- **background**：近白 + 极弱冷灰。
- **card / sidebar**：白或更浅灰分层。
- **primary**：同色相略深，保证浅色背景对比。
- **border / input**：浅灰实线，不过重。
- **destructive** 保持可读红，不追求品牌化。

#### chart 色

- 保留偏蓝紫序列，与 primary 色相协调；不引入高饱和彩虹色。

### 4.2 形状

- `--radius`：`0.625rem` → **`0.75rem`**
- 继续依赖现有 `--radius-sm/md/lg/...` 计算链，组件自动变圆润。

### 4.3 字体与基础排版

- 字体栈保持 Windows 友好：`Segoe UI` 优先（现有 `App.css` 已接近）。
- `body` 字号/行高微调为更桌面化：**15px / 1.5**（当前 16px / 1.6）。
- 不改组件内部字号 class。

### 4.4 启动底色

- `html` / `#root` 当前硬编码 `#0f0f0f`，改为与新 dark `background` 视觉一致的近似值，减少启动闪白/闪色。

### 4.5 默认主题

- `ThemeProvider` 的 `defaultTheme`：`system` → **`dark`**（仅 localStorage 无值时）。
- preferences 加载失败/文件不存在时的默认 `theme`：`system` → **`dark`**。
- **不迁移**已有用户已保存的主题偏好。

## 5. 落地文件

| 文件                               | 变更                                     |
| ---------------------------------- | ---------------------------------------- |
| `src/theme-variables.css`          | 更新 `:root`、`.dark` 语义色；`--radius` |
| `src/App.css`                      | 启动底色；body 字号/行高微调             |
| `src/components/ThemeProvider.tsx` | `defaultTheme` 默认值改为 `dark`         |
| `src/services/preferences.ts`      | 默认 preferences 中 `theme: 'dark'`      |

可选（仅当实现时发现测试/快照依赖旧默认）：

- 相关单元测试中的 theme 期望值。

## 6. 实现约束

- 只改全局 token 与默认值，**不改**页面 JSX 结构。
- 不新增布局级 shadow class；如需 shadow CSS 变量，仅预留且本轮可不被组件引用。
- 保持 shadcn 语义命名，避免引入非标准 token 导致组件失效。
- 中文 IME、Radix focus 等现有 UI 规则不受影响。

## 7. 验证计划

1. 手动：深/浅切换，检查侧栏、卡片、对话框、按钮、输入框、焦点环、toast。
2. 手动：Windows 冷启动，观察是否仍有明显白闪。
3. 命令：`npm run check:all`。
4. 回归：Preferences → Appearance 主题选择仍可切换 light/dark/system 并持久化。

## 8. 风险与回退

| 风险                                   | 缓解                                             |
| -------------------------------------- | ------------------------------------------------ |
| primary 从中性灰变蓝，部分按钮观感变化 | 依赖语义 token；目视检查 primary/secondary/ghost |
| 硬编码旧灰色局部不协调                 | 本轮不扫全库；明显问题定点修                     |
| 默认 dark 影响新用户/测试              | 仅默认值；老用户偏好保留                         |

回退：还原上述 4 个文件即可。

## 9. 后续可选（不在本 spec）

- **方案 B**：壳层轻量亚克力（`backdrop-blur` + 半透明 surface）。
- **方案 C**：读取 Windows 强调色动态映射 CSS 变量。
- 列表/卡片密度与空状态文案视觉（需动组件，超出 token 范围）。
