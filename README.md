# DroidGear

[English](README_EN.md)

[Factory Droid](https://factory.ai) / [Claude](https://claude.ai) / [OpenClaw](https://openclaw.ai) / [Codex](https://github.com/openai/codex) / [OpenCode](https://opencode.ai) / [Hermes Agent](https://hermes-agent.nousresearch.com) / [Pi](https://pi.dev) / [Dsh (DeepSeek Harness)](https://deepseek.ai) 的桌面增强管理工具。

支持自定义 AI 模型（BYOK）、嵌入式终端、会话与 Specs 管理、MCP 服务器配置等。

## 安装说明

### macOS

由于应用未经 Apple 签名，下载后可能被 Gatekeeper 阻止运行。请执行以下命令解除限制：

```bash
xattr -cr /Applications/DroidGear.app
```

### Windows / Linux

直接运行安装程序即可。

## 功能特性

### 自定义模型管理

- **多服务商支持** - 配置来自 Anthropic、OpenAI 或任何通用 Chat Completion API 的模型
- **可视化模型管理** - 通过拖拽添加、编辑、删除和重新排序自定义模型
- **模型收藏** - 收藏常用 Droid 模型，收藏管理器限制内置模型并保持对话框顺序
- **批量操作** - 复制模型、筛选和批量删除、设置默认模型
- **模型注册表** - 在偏好设置中浏览和搜索内置模型注册表中的可用 AI 模型
- **API 模型发现** - 直接从服务商 API 获取可用模型列表，自动生成 ID 和显示名称
- **配置导入导出** - 支持配置的导入导出和批量管理
- **多平台支持** - 支持 sub2api、antigravity、DeepSeek 等多种 API 代理平台
- **Ollama 支持** - 支持 Ollama 频道，自动检测本地 Ollama 服务
- **服务商模板** - 内置 OpenAI、Gemini 等服务商模板，快速配置

### 嵌入式终端

- **内置终端** - 集成终端支持，可保存和恢复终端状态
- **自定义配置** - 支持自定义字体、强制深色模式
- **便捷操作** - 选中即复制、OSC 9 通知、派生子窗口
- **外部终端启动** - macOS 支持通过 Ghostty、iTerm2 启动 CLI 工具
- **快捷键支持** - Cmd/Ctrl+1~0 切换终端、Cmd/Ctrl+W 关闭标签页、Cmd/Ctrl+Shift+[ 切换标签
- **代码片段** - 终端页面支持代码片段功能

### Droid 会话管理

- **会话查看** - 查看和管理 Droid 会话，支持删除会话
- **多种视图** - 列表/分组视图切换，支持隐藏空会话
- **跟随模式** - 会话跟随模式，支持切换思考展开状态
- **云端同步** - 云端会话同步开关

### Droid 配置管理

- **多配置文件** - 支持管理和切换多个 Factory Droid 配置文件（settings.json）
- **关联本地配置** - 关联本地 JSON 文件作为启动配置（`--settings`），文件保留在原位置（GUI 与 TUI）
- **信任文件夹** - 在 GUI 与 TUI 中查看和管理 Droid 信任文件夹
- **终端偏好** - 为每个配置文件独立设置终端偏好
- **面板刷新** - 切换配置文件时自动刷新相关面板

### Missions 管理

- **模型配置** - 配置 Missions 的 Worker 模型和 Validation Worker 模型
- **推理力度** - 为每个模型独立设置推理力度（none/low/medium/high/max），模型卡片显示 1M 上下文徽章

### Specs 规格文件管理

- **文件浏览** - 查看 `~/.factory/specs` 目录下的规格文件
- **Markdown 渲染** - 支持 Markdown 格式渲染
- **文件操作** - 重命名、删除、另存为、复制完整路径
- **编辑模式** - 支持规格选择和编辑模式
- **导出功能** - 支持导出规格项

### MCP 服务器管理

- **预设配置** - 内置 MCP 预设（包含 exa 等）
- **服务器管理** - MCP 服务器配置管理

#### 百智工具包（Baizhi MCP）

GUI 的 MCP「快速添加」卡片或 TUI MCP 列表的 `b` 键会打开百智预设确认；选择和取消不写入配置、不连接服务。确认后保存默认停用的 HTTP 条目，固定 URL 为 `https://agent-toolkit.app.baizhi.cloud/mcp`，Header 仅为字面引用 `Authorization: Bearer ${BAIZHI_API_KEY}`。此预设流程不接收 Key、不查询 BAIZHI_API_KEY 环境变量或展开真实 Key；保存会读取原配置以保留其他记录。

请通过[百智工具包使用入口](https://agent-toolkit.app.baizhi.cloud/)了解独立账号及费用，另在**启动 Factory 的环境**中设置 `BAIZHI_API_KEY` 后再显式启用。不要将真实 Key 填入预设或 URL。根据 [Factory 官方 MCP 文档](https://docs.factory.com/harness/mcp)，Header 中的环境变量在连接时展开，未设置变量会使连接失败，展开结果不回写文件；这些是官方声明，此变更未验证真实 Factory 连接、模型调用或 GUI/shell 环境继承。

配置仍写入普通 `~/.factory/mcp.json`；保存的是非秘密引用，不是加密存储或钥匙串。百智专属写入入口在新条目的原始 JSON 中保存 `oauth: false`，对应 Factory 官方对 Header 认证的推荐。启用/停用切换会保留该选项；在现有通用编辑器保存条目可能丢掉未建模的 `oauth` 字段，编辑后请核对配置。这里未验证真实 Factory 的 OAuth 行为。启用后，工具参数发送给百智，返回内容可能进入所选 AI 模型。停用或删除本地条目不等于撤销 Key、远端任务或费用。

### OpenCode 支持

- **AI 开发集成** - OpenCode 工具集成
- **配置管理** - 服务商/认证配置的加载和保存

### OpenClaw 支持

- **AI 开发集成** - OpenClaw 工具集成
- **配置管理** - 服务商配置的加载和保存
- **安装辅助** - 内置 macOS/Linux/Windows 安装命令

### Claude 支持

- **Claude 设置文件管理** - 多设置文件（Global + 自定义）切换与编辑，结构化 Provider / 模型 / Reasoning / Thinking / 通用设置 / 权限分段
- **设置文件操作** - 新建、删除、复制、从实时配置加载、合并到全局、变更预览，以及临时单次运行
- **设为活动与脏状态跟踪** - 可将设置文件设为活动，未保存的修改有脏状态提示（TUI）
- **合并应用** - 将设置文件中的 env 与 permissions 浅合并到全局设置
- **从渠道导入** - 一键从已配置渠道导入 Provider 配置（base URL、API Key、默认模型）
- **模型拉取** - 使用已配置 Provider 的 Bearer Token 拉取模型 ID，主模型与 Small 模型可从列表选择，仍可手动输入（GUI + TUI）
- **自动压缩窗口** - 在 GUI 与 TUI 中设置根级 `autoCompactWindow`（100,000–1,000,000 tokens）。留空则由 Claude Code 按模型选择窗口；`/autocompact` 按模型保存的窗口与环境变量 `CLAUDE_CODE_AUTO_COMPACT_WINDOW` 仍然优先
- **运行时核心** - 启动时通过运行时覆盖层加载设置，从不修改实时配置文件
- **GUI/TUI 功能对齐** - 桌面版与 TUI 版的设置功能与语义保持一致（Small 模型留空时使用 Claude 默认的 Haiku 5.5，跟随主模型需显式开启）

### Codex 支持

- **Codex CLI 集成** - Codex 配置 Profile 管理
- **认证配置** - 官方登录与 BYOK 认证配置文件的保存、切换及冲突检测
- **即时应用** - 编辑当前活动的 Profile 时立即应用（含切换 Provider），无需手动应用
- **模型目录同步** - 应用 DeepSeek V4 / MiMo 模型时自动同步 `~/.codex/model-catalogs/` 下的分族模型目录（`model_catalog_json` 随模型族切换）

- **配置管理** - 认证与 `config.toml` 的加载和保存（`~/.codex`）
- **管理入口** - 在 Codex 入口下提供 MCP 服务器 / 会话 / 终端 管理子页（会话页支持按项目分组浏览、思考过程与 Markdown 渲染、会话删除及 Provider 归属切换）

### Hermes Agent 支持

- **配置管理** - Hermes Agent YAML Profile 配置管理
- **推理强度** - 为每个 Hermes Profile 独立设置推理强度（none/minimal/low/medium/high/xhigh/max/ultra）
- **渠道导入** - 支持从渠道导入 Hermes Agent 配置

### Pi 支持

- **Pi 编码代理集成** - Pi（pi.dev）自定义模型配置管理
- **Provider/Model 管理** - 支持配置多个 Provider 及其模型（baseUrl, api, apiKey, headers, compat 等）
- **连接测试** - 测试 Pi Provider 连通性
- **Profile 管理** - 多 Profile 支持，一键应用到 `~/.pi/agent/models.json`
- **实时配置读取** - 支持从 Pi 的 live config 加载现有配置
- **模型注册表驱动** - 从内置模型注册表丰富 Pi Provider 模型，支持与 Provider 无关的思考级别映射。Anthropic 编码为 `anthropic-adaptive` 的模型（如 `claude-opus-5-5`）会写入 `compat.forceAdaptiveThinking`，已显式关闭的配置会保留

### Dsh（DeepSeek Harness）支持

- **Provider 配置管理** - 管理新版 DSH 多 Profile 的 `cordis.patch.yml` 中 `llm-pi-ai.providers`（displayName、baseURL、apiKeyEnv、api、compat、模型列表）：支持官方 desktop / web 两种运行时 profile（默认 desktop），写入时保留其余 patch 条目与 `!!js` 表达式；无 profile 时回退到旧版 `~/.dsh/settings.yaml`，读取时按 旧版 settings.yaml → profile patch → home patch 顺序合并
- **密钥值管理** - 在 `~/.dsh/.credentials.yaml` 中配置 API 密钥值（0600 权限）。Provider 的凭据引用名默认由 ID 推导为 `<PROVIDER_ID>_API_KEY`，也可自定义以复用已有环境变量；存储值优先于环境变量与 `.env`
- **官方 DeepSeek API Key** - 在模型页与 TUI（按 `o`）为不通过 DSH 登录的用户配置官方密钥，写入 `llm-deepseek.apiKeyEnv`（默认 `DEEPSEEK_API_KEY`）对应的凭据；清除后回退到环境变量
- **可视化配置** - desktop / web Profile 的合成配置分为概览、模型、通用设置、Agent、网络、高级与诊断七个页签；TUI 按 `c` 查看并编辑字段、切换条目禁用。写入 Profile 的 `cordis.patch.yml`，保留注释与 `!!js`，密钥脱敏，并用修订哈希避免覆盖并发修改
- **默认思考强度与默认模型** - pi-ai 路由可设置默认思考级别（`off` 至 `max`，该路由上的模型都支持时建议 High）；可将某个模型标为默认并写入 `agent-default-model`（TUI 按 `g`）
- **模型输入类型** - 按模型查看并编辑文本 / 图像输入；未手动固定前按注册表建议填充，避免自定义路由被 DSH 当成仅文本
- **渠道导入** - 从渠道导入 Provider（自动填充 ID、Base URL、密钥与 API 类型）
- **模型拉取** - 配置 URL 与密钥后从服务商 API 拉取模型列表，多选添加
- **会话历史** - 浏览 `~/.dsh/sessions` 会话日志（GUI + TUI）：列表/按项目分组、思考内容渲染、实时追踪、隐藏空会话与删除；连续的纯思考步骤合并为一个可展开的「思考过程」块
- **注册表自动适配** - 按模型 ID 自动填充名称、上下文窗口、最大 Token 与推理强度（reasoningEfforts）

### GitHub Copilot 支持

- **BYOK Profile 管理** - 管理 GitHub Copilot BYOK 配置（OpenAI / Anthropic 提供商、模型与 Token 限制），支持创建、复制、删除、应用与加载已应用配置
- **渠道导入** - 从渠道导入 Provider 与模型，自动规范 Base URL（OpenAI 补 `/v1`、Anthropic 去 `/v1`）
- **官方订阅模式** - 清除 BYOK 覆盖，使用 Copilot CLI 现有登录或认证 Token
- **临时运行** - 一键以所选配置启动 Copilot（进程级环境变量 + 显式 `--model` 参数）
- **独立启动器** - 附带 `copilot-local.sh` / `copilot-local.cmd` 启动器，无需 GUI 即可用已导出的配置启动 Copilot

### OMP（Oh My Pi）支持

- **自定义 Provider** - 按官方 `~/.omp/agent/models.yml` 读写自定义 Provider 与模型（GUI 与 TUI 增删改），支持从渠道更新模型和 Provider，并保留其他 Provider 与未知字段
- **模型角色** - 管理 `~/.omp/agent/config.yml` 中的模型角色，并可从当前配置加载
- **会话历史** - 浏览 `~/.omp/agent/sessions` 的 JSONL 日志（GUI）：列表或按项目分组、渲染当前分支与思考内容、跟随新消息、隐藏空会话与删除，滚动时分批加载

### 其他功能

- **自动更新** - 版本检查、自动更新提示和下载进度显示
- **窗口状态** - 保存和恢复窗口状态
- **退出保护** - 有未保存更改时关闭会提示警告
- **跨平台** - 支持 macOS、Windows 和 Linux

## TUI 版本（无头终端）

DroidGear TUI 是专为 SSH 和无桌面环境设计的终端界面版本，与桌面版共享相同的配置文件和核心功能。

### 安装

从 [Releases](https://github.com/Sunshow/droidgear/releases) 下载对应平台的 `droidgear-tui` 二进制文件：

- macOS (Apple Silicon): `droidgear-tui-*-aarch64-apple-darwin.tar.gz`
- macOS (Intel): `droidgear-tui-*-x86_64-apple-darwin.tar.gz`
- Linux: `droidgear-tui-*-x86_64-unknown-linux-gnu.tar.gz`
- Windows: `droidgear-tui-*-x86_64-pc-windows-msvc.zip`

解压后将二进制文件放到 PATH 路径下（如 `/usr/local/bin`）。

### 运行

```bash
# 使用默认配置（读取 ~/.factory 和 ~/.droidgear）
droidgear-tui

# 指定自定义 HOME 路径（容器/测试场景）
droidgear-tui --home /path/to/custom/home

# 一次性运行 Codex profile（当前终端直接接管运行）
droidgear-tui run codex <profile-id>
```

### 功能支持

TUI 版本支持以下配置管理功能：

- **分组导航**：与 GUI 两级侧边栏对应的分组导航，支持功能列表界面、面包屑标题与模块选择器标签
- **Factory 配置**：自定义模型管理、默认模型设置
- **MCP 服务器**：增删改查、启用/禁用、导入导出
- **Claude 设置文件**：多设置文件管理、结构化字段编辑、合并应用、设为活动、跳过权限运行、脏状态跟踪、临时单次运行、模型拉取、自动压缩窗口
- **Codex Profile**：配置文件管理、变更预览、一键应用
- **OpenCode Profile**：Provider/Auth 配置管理
- **OpenClaw Profile**：配置管理与应用（含 Subagents/Helpers 导航入口）
- **Hermes Profile**：配置管理与应用
- **Pi Profile**：Provider/Model 配置管理与应用
- **Dsh**：Profile 可视化配置（按 `c`）、`llm-pi-ai.providers` 与凭据管理（含渠道导入、模型拉取、官方 DeepSeek API Key、默认思考强度与默认模型）
- **Copilot Profile**：BYOK Profile 配置管理与应用（含渠道导入、临时运行）
- **OMP**：自定义 Provider（`models.yml`）增删改与渠道导入，以及 `config.yml` 模型角色
- **Sessions**：会话浏览与管理
- **Paths**：路径覆盖配置（适配服务器环境）
- **Channels**：代理平台与凭据管理

### 基本操作

- `↑/↓` 或 `j/k`：上下移动
- `Enter`：进入/确认
- `Esc`：返回/取消
- `Tab`：切换焦点区域
- `Ctrl+S`：预览变更（编辑页面）
- `y/N`：确认应用变更
- `q`：退出（主界面）

### 配置文件

TUI 版本与桌面版共享配置文件：

- Factory 配置：`~/.factory/settings.json`
- MCP 配置：`~/.factory/mcp.json`
- DroidGear 配置：`~/.droidgear/`
- Codex 配置：`~/.codex/`
- OpenCode 配置：`~/.config/opencode/`
- OpenClaw 配置：`~/.openclaw/`
- Hermes 配置：`~/.hermes/`
- Pi 配置：`~/.pi/agent/`

详细设计文档见 [docs/developer/tui-design.md](docs/developer/tui-design.md)

## 配置说明

DroidGear 读写 `~/.factory/settings.json` 文件：

```json
{
  "customModels": [
    {
      "model": "your-model-id",
      "displayName": "我的自定义模型",
      "baseUrl": "https://api.provider.com/v1",
      "apiKey": "YOUR_API_KEY",
      "provider": "generic-chat-completion-api",
      "maxOutputTokens": 16384
    }
  ]
}
```

### 支持的服务商

| 服务商    | 值                            |
| --------- | ----------------------------- |
| Anthropic | `anthropic`                   |
| OpenAI    | `openai`                      |
| 通用 API  | `generic-chat-completion-api` |

## 开发指南

### 前置要求

- Node.js 20+
- Rust（最新稳定版）
- 平台特定依赖：https://tauri.app/start/prerequisites/

### 启动开发

```bash
npm install
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

## 技术栈

- **前端**: React 19, TypeScript, Vite, Tailwind CSS, shadcn/ui
- **后端**: Tauri v2, Rust
- **状态管理**: Zustand

## 隐私声明

DroidGear 重视您的隐私安全。您的用户名、密码、API 密钥等敏感数据仅存储在本地设备，不会上传至任何服务器。

## 更新日志

查看完整更新日志请访问 [CHANGELOG.md](CHANGELOG.md)

## 致谢

本项目基于 [tauri-template](https://github.com/dannysmith/tauri-template) 开发，感谢 Danny Smith 提供的优秀模板。

## 许可证

[MIT](LICENSE.md)
