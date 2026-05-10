# Claude 02: Apply 与 Live Settings Merge

## Overview

这一步把 Claude profile 真正应用到 live Claude config：

- 读取当前 `settings.json`
- 只修改受管键
- 保留无关配置
- 正确清理冲突键

同时补上：

- config status
- current config readback

这样 GUI/TUI 才能做：

- Apply
- Load From Live Config
- 状态提示

## Goals

- 实现 Claude live config status 查询
- 实现 Claude current config 读取
- 实现 Apply 到 `<CLAUDE_CONFIG_DIR>/settings.json`
- 严格限制只改受管字段
- 明确冲突 env 的清理语义

## Non-Goals

- 不在这一步实现 Temporary Run
- 不在这一步做 launcher shim
- 不在这一步开放 GUI/TUI 编辑页

## Managed Keys

Apply 只应管理这些字段：

- top-level
  - `alwaysThinkingEnabled`
- `env`
  - `ANTHROPIC_BASE_URL`
  - `ANTHROPIC_AUTH_TOKEN`
  - `ANTHROPIC_MODEL`
  - `ANTHROPIC_DEFAULT_HAIKU_MODEL`
  - `CLAUDE_CODE_EFFORT_LEVEL`
  - `CLAUDE_CODE_DISABLE_THINKING`
  - `MAX_THINKING_TOKENS`
  - `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`

Apply 还应清理这些冲突键：

- `ANTHROPIC_API_KEY`
- `CLAUDE_CODE_USE_BEDROCK`
- `CLAUDE_CODE_USE_VERTEX`
- `CLAUDE_CODE_USE_FOUNDRY`
- `ANTHROPIC_BEDROCK_BASE_URL`
- `ANTHROPIC_BEDROCK_MANTLE_BASE_URL`
- `ANTHROPIC_VERTEX_BASE_URL`
- `ANTHROPIC_VERTEX_PROJECT_ID`
- `ANTHROPIC_FOUNDRY_BASE_URL`
- `ANTHROPIC_FOUNDRY_RESOURCE`
- `ANTHROPIC_FOUNDRY_API_KEY`
- `CLAUDE_CODE_PROVIDER_MANAGED_BY_HOST`

## Important Decision

这轮不要主动清理 top-level：

- `model`
- `effortLevel`

原因：

- Claude profile 这轮已经决定走 env contract
- 用户原有的 live top-level 默认值应继续保留为非受管层
- 这样对现有 Claude 使用习惯侵入更小

## Live Config Readback Rules

`read_claude_current_config()` 需要兼容两种来源：

### 1. 受管 env 优先

如果 live settings 里已有：

- `env.ANTHROPIC_MODEL`
- `env.CLAUDE_CODE_EFFORT_LEVEL`

则按受管 env 读回。

### 2. top-level 作为导入回退

如果受管 env 不存在，但用户 live settings 有：

- top-level `model`
- top-level `effortLevel`

则读回时应把它们当作 import fallback。

否则“Load From Live Config”会无端丢掉用户已有默认配置。

## Suggested File Changes

- `src-tauri/crates/droidgear-core/src/claude.rs`
  - 如果文件过大，可拆出 helper
- `src-tauri/crates/droidgear-core/src/json.rs`
  - 如需复用 JSON merge / atomic write helper
- `src-tauri/src/commands/claude.rs`
- `src/lib/tauri-bindings.ts`
  - 通过 `npm run rust:bindings`

## Implementation Steps

### 1. 新增 live config status 类型

建议至少包含：

- `settingsExists`
- `settingsPath`
- `configDir`
- parse error / readable 状态

### 2. 新增 current config readback

需要把 live settings 映射回：

- `baseUrl`
- `bearerToken`
- `model`
- `smallModel`
- `reasoningEffort`
- `thinkingMode`

### 3. 实现 Apply merge

核心要求：

- 先读现有 `settings.json`
- 只动受管 top-level / `env`
- 受管字段为 `null` / unset 时，从 JSON 中删除对应 key
- 无关字段保持原样

### 4. 处理 malformed settings

如果 user settings JSON 本身损坏：

- Apply 不能 silent 覆盖
- 必须返回明确错误

## Acceptance Criteria

- Apply 可把 profile 写入 live Claude settings
- Apply 只改受管字段
- Apply 不会破坏无关字段
- 冲突 env 会被正确清理
- current config readback 可从 live settings 正确导入

## Tests

建议至少覆盖：

- `settings.json` 不存在时的 apply
- 已有无关字段时的 merge-preserve
- `smallModelUsesMainModel=true` 的写入
- `thinkingMode=on/off/inherit` 的写入与清理
- `reasoningEffort=low/medium/high/max/null` 的写入与清理
- malformed JSON 时的错误路径
- readback 对 env 优先 / top-level fallback 的行为

## Dependencies

- 依赖 [task-x-claude-01-core-profile-storage-and-path-plumbing.md](./task-x-claude-01-core-profile-storage-and-path-plumbing.md)
