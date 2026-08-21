# TUI（Headless）版本设计：抽离核心库 + 终端交互

目标：在**没有桌面环境**的 Linux（SSH 终端）上提供 DroidGear 的主要能力，同时保证桌面版逻辑不分叉、不回归。

本设计采用：**把 Rust 侧的业务逻辑抽离成 `core` 库**，桌面端（Tauri commands）与 TUI 端复用同一套核心逻辑；并用单元测试/特征化测试锁住行为，避免重构破坏现有功能。

## 目标与非目标

### 目标（覆盖你提到的常用项）

| 能力                                 | 现状（桌面版）                                                                                  | TUI 目标                                               |
| ------------------------------------ | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| Factory 配置（模型/默认模型/开关项） | `~/.factory/settings.json`（见 `src-tauri/src/commands/config.rs`）                             | 列表/增删改/复制/排序、设置默认、导入导出、变更预览    |
| MCP 服务器管理                       | `~/.factory/mcp.json`（见 `src-tauri/src/commands/mcp.rs`）                                     | 列表/增删改/启用禁用、预设一键添加、导入导出、变更预览 |
| Codex Profile 管理与一键应用         | `~/.droidgear/codex/**` + `~/.codex/**`（见 `src-tauri/src/commands/codex.rs`）                 | Profile CRUD、显示当前 live 配置、Plan/Preview/Apply   |
| OpenCode Profile 管理与一键应用      | `~/.droidgear/opencode/**` + `~/.config/opencode/**`（见 `src-tauri/src/commands/opencode.rs`） | Profile CRUD、Plan/Preview/Apply（含 JSONC 优先规则）  |
| OpenClaw Profile 管理与一键应用      | `~/.droidgear/openclaw/**` + `~/.openclaw/**`（见 `src-tauri/src/commands/openclaw.rs`）        | Profile CRUD、Plan/Preview/Apply（含 deep-merge 规则） |
| Sessions 管理                        | `~/.factory/sessions/**`（见 `src-tauri/src/commands/sessions.rs`）                             | 项目/会话列表、详情查看、删除                          |
| Paths 覆盖（适配服务器/容器）        | `~/.droidgear/settings.json`（见 `src-tauri/src/commands/paths.rs`）                            | 查看有效路径、设置/重置路径、对其它模块生效            |
| Channels（代理平台/凭据/Token 拉取） | `~/.droidgear/channels.json` + `~/.droidgear/auth/**`（见 `src-tauri/src/commands/channel.rs`） | Channel CRUD、凭据管理、拉取 token（必要时）           |

### 非目标（TUI 不必等价桌面）

- 桌面端专有：窗口状态、自动更新 UI、原生通知、全局快捷键、嵌入式终端标签页等（依赖 Tauri/WebView/桌面插件）。
- Specs 编辑：TUI 中只保留“浏览/打开到 `$EDITOR`”即可；不做复杂 Markdown 渲染器。

## 总体架构：一个 core，两种前端

核心原则：**业务逻辑不在 Tauri command 里“长出来”**，Tauri 与 TUI 都只是 UI/交互层。

建议把 `src-tauri` 变成 Cargo workspace（同一个 `Cargo.toml` 同时是 package + workspace root），新增：

- `src-tauri/crates/droidgear-core/`（库）
  - 无 `tauri` 依赖（可以有 `serde/serde_json/toml/reqwest/notify` 等）
  - 提供“读取 → 计算变更 → 预览 → 原子写入”的 API
- `src-tauri/`（现有 Tauri app crate）
  - commands 变成 thin wrapper：参数校验、调用 core、把结果返回给前端
- `src-tauri/crates/droidgear-tui/`（二进制）
  - 终端 UI：推荐 `ratatui + crossterm`（对 SSH/tmux/screen 兼容性好）
  - 只调用 `droidgear-core`，不依赖 Tauri

## “变更预览”作为一等能力：ChangeSet

为了支持你提到的“变更预览(diff) + 一键应用 + 导入导出”，建议在 core 里统一成：

- **Plan**：不落盘，计算会写哪些文件、写入后内容是什么
- **Preview**：把 Plan 渲染成可展示的 diff（TUI/GUI 都可用）
- **Apply**：按 Plan 原子写入（写 tmp → rename），并可选创建 `.bak` 备份

抽象建议：

- `ChangeSet { changes: Vec<FileChange> }`
- `FileChange { path, before: Option<Vec<u8>>, after: Vec<u8>, kind }`
  - `kind` 用于决定 preview 方式：JSON/TOML/Text/Binary
  - `before` 为空表示新建文件

好处：

- GUI/TUI 的“预览/确认/应用”交互一致
- 单元测试可以直接断言 `after` 内容，避免 UI 细节影响测试
- 后续实现“导出到单个 zip / 目录”也自然（仍是 ChangeSet）

## core API 边界（按域拆分）

core 以“域模块”提供能力，尽量保持与现有 `src-tauri/src/commands/*.rs` 的结构一致，减少重构风险：

- `paths`：解析 `~/.droidgear/settings.json` 的 `configPaths`，输出 effective paths
- `factory_settings`：`~/.factory/settings.json` 的读写与校验（customModels、默认模型、开关项）
- `mcp`：`~/.factory/mcp.json` 的 CRUD/toggle
- `channels`：`~/.droidgear/channels.json` 与 `~/.droidgear/auth/*.json`（凭据）管理；以及 token 拉取（需要网络）
- `codex/opencode/openclaw`：profile CRUD + plan/apply（写入到 live 配置）
- `sessions/specs`：文件系统浏览与安全检查（只读为主；删除/重命名/写入需要明确确认）

实现上要避免的坑（重构时常见回归点）：

- ❌ core 内部直接用 `dirs::home_dir()`：测试很难稳定、也不利于容器化
- ✅ core 用 `Environment { home_dir, now_fn, ... }` 或显式传入根路径（测试用临时目录）
- ❌ core 直接写文件（无预览）：无法做 diff、也难测
- ✅ core 先产出 ChangeSet，再由调用方决定 apply

## TUI 交互（产品层面）

建议的主导航（不需要鼠标）：

- 左侧模块列表：Factory / MCP / Codex / OpenCode / OpenClaw / Sessions / Paths / Channels
- 右侧主内容：列表 → 详情/编辑表单 → Preview → Confirm Apply
- 通用操作：
  - `Enter`：进入/确认
  - `Esc`：返回/取消
  - `Ctrl+S`：在编辑页触发 Preview（不直接写盘）
  - `y/N`：应用变更确认
  - `$EDITOR`：打开复杂文本或 JSON（Specs、OpenClaw 大段配置等）

## 敏感信息（Secrets）处理

TUI 的默认行为应尽量“安全”，避免 SSH 录屏、滚屏、粘贴误泄露：

- 默认打码显示：API Key、密码、token、Authorization header 等（列表、详情、diff preview 都打码）
- 显示明文需显式动作：例如 `Reveal`（再确认一次），并在退出页面时自动恢复打码
- 导出默认不含 secrets：导出配置时默认剔除敏感字段；需要“包含 secrets”时必须显式选择

对应单元测试建议：

- `redact_*` 系列函数：输入包含 secrets 的结构 → 预览输出不应包含明文
- `plan/apply` 不受影响：预览打码不改变最终写盘内容（ChangeSet 的 `after` 仍是完整配置）

## 测试策略（重构与新增 TUI 的“保险丝”）

目标：在抽离 core 与实现 TUI 的过程中，**保证与现有桌面版完全一致的落盘结果**（同输入 → 同输出文件内容）。

### 1）先做“特征化测试”（锁定现有行为）

在移动代码前，对以下高风险逻辑增加 Rust 单元测试（推荐放在未来的 `droidgear-core`，但可先在 `src-tauri` 内写测试以锁住行为，再迁移）：

- Codex apply：只替换模型相关字段，保留其它 TOML 配置（见 `apply_codex_profile`）
  - 断言：`model_provider/model/model_reasoning_effort/model_providers` 发生变化
  - 断言：其它 key（如 `projects`、`network_access` 等）保持原样
  - 断言：custom provider 将 API key 写入 `config.toml` 的 `experimental_bearer_token`，并设置 `requires_openai_auth = true`；不再写入 `auth.json` 的 `OPENAI_API_KEY`（空 token 不写；残留 key 从 auth.json 剥离）
- OpenClaw deep-merge：`REPLACE_PATHS` 上“整段替换”，其它路径 deep merge（见 `deep_merge_with_replace`）
- OpenCode apply：`opencode.jsonc` 优先于 `opencode.json`，合并规则为“provider/auth shallow merge”（见 `apply_opencode_profile`）
- Paths：`save_config_path/reset_config_path` 对 `~/.droidgear/settings.json` 的读写与 key 映射规则
- MCP：`toggle`/CRUD 的 JSON 结构不变形（`mcpServers` map）
- Factory settings：`customModels` 的序列化保持字段命名（camelCase），并保留 settings.json 其它字段不丢失

测试实现建议：

- 使用 `tempfile::TempDir` 建立临时 HOME，把所有配置文件写在 temp 里
- 用“输入文件内容（fixture）→ 调用 plan/apply → 读取结果 → 与期望字符串比较”的方式断言
- 对 TOML/JSON 结果建议比较**最终写盘字符串**（而不是解析后结构），这样能捕捉意外格式变化

### 2）core 抽离后的单元测试（主要阵地）

core 的测试以“纯函数 + 文件计划”为主：

- `plan_*` 返回的 `ChangeSet` 内容正确（路径集合、after 内容、kind）
- `apply_changeset` 的原子写入行为正确（至少验证最终文件内容；rename 失败的模拟可选）

### 3）TUI 测试（只测“交互逻辑”，不测业务）

TUI 尽量做 thin UI：业务都在 core，所以 TUI 测试只需要：

- “路由/状态机”测试：例如从列表进入编辑，生成 preview，再确认 apply（不需要真实终端，ratatui 提供测试后端）
- `--dry-run`（若提供）输出与 ChangeSet 一致

网络相关（token 拉取、模型发现）建议只做：

- 单元测试：对请求构造/解析函数做纯测试
- 集成测试可选：在 CI 里用 mock server（如 `httpmock`/`wiremock-rs`），避免依赖真实外网

## 实施顺序（保证不破坏桌面版）

1. 先补齐“特征化测试”，让当前桌面版的关键逻辑有回归保护。
2. 抽离 `droidgear-core`，让现有 Tauri commands 变成 thin wrapper，并保持测试全绿。
3. 新增 `droidgear-tui`，复用 core，先实现你常用的模块（Factory/MCP/Profiles/Paths/Sessions/Channels），Specs 只做浏览与 `$EDITOR` 打开。

完成后，桌面版与 TUI 共享同一套 core，后续新增功能只需要在 core 增加 plan/apply，再分别在 GUI/TUI 加页面即可。
