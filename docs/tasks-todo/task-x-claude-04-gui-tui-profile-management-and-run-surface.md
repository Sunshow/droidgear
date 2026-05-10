# Claude 04: GUI / TUI Profile Management 与 Run Surface

## Overview

在 core、apply、temp run 都稳定后，再开放用户入口：

- 桌面 GUI
- 交互式 TUI
- `droidgear-tui run claude`

这一步的目标是让 Claude 成为和 Codex 同级的一条正式工具线，而不是只存在后端命令。

## Goals

- 新增 Claude Zustand store
- 新增 Claude GUI 页面与组件
- 新增 Claude TUI 页面
- 新增 Claude TUI `run` / `--list`
- 接入 Tauri commands 与 bindings
- 增加 i18n 文案

## Non-Goals

- 不在这一步做 advanced capability preset
- 不在这一步做 `_SUPPORTED_CAPABILITIES` 编辑 UI
- 不在这一步做 Claude 自带 provider 专项面板

## Suggested File Changes

- `src/store/claude-store.ts`
- `src/components/claude/`
  - `ClaudeConfigPage.tsx`
  - `ClaudeFeatureList.tsx`
  - `ConfigStatus.tsx`
  - 如需要可拆出 provider / model 子卡片
- `src/components/layout/LeftSideBar.tsx`
- `src/App.tsx`
  - 视当前导航结构而定
- `src-tauri/src/commands/claude.rs`
- `src/lib/tauri-bindings.ts`
  - 通过 `npm run rust:bindings`
- `locales/en.json`
- `locales/zh.json`
- `src-tauri/crates/droidgear-tui/src/app.rs`
- `src-tauri/crates/droidgear-tui/src/ui.rs`
- `src-tauri/crates/droidgear-tui/src/tui/mod.rs`
- `src-tauri/crates/droidgear-tui/src/tui/keys_*.rs`
- `src-tauri/crates/droidgear-tui/src/tui/modal.rs`
- `src-tauri/crates/droidgear-tui/src/main.rs`
- `src-tauri/crates/droidgear-tui/src/tui/utils.rs`

## GUI Requirements

### 必备操作

- profile selector
- create
- duplicate
- delete
- save
- apply
- run
- load from live config

### 必备编辑字段

- `name`
- `description`
- `baseUrl`
- `bearerToken`
- `model`
- `smallModelUsesMainModel`
- `smallModel`
- `reasoningEffort`
- `thinkingMode`

### 明确不要出现的错误入口

- 不要暴露 `xhigh`
- 不要暴露 `API Key`
- 不要暴露认证 `inherit`

## TUI Requirements

Claude 不能只做 GUI。

至少需要：

- Claude profile list screen
- Claude profile detail/edit screen
- Apply
- Run
- create / duplicate / delete
- `droidgear-tui run claude --list`
- `droidgear-tui run claude <selector>`

selector 规则建议对齐 Codex：

- index
- exact name
- exact id

## Shared Helper Guardrail

这一步不能直接复用当前“所有 Claude 都支持 xhigh”的 shared helper 结果来渲染 Claude UI。

需要二选一：

1. Claude 页面使用 Claude 自己的固定 option list
2. 或先修正 shared helper，再安全复用

这条如果不处理，前端很容易把错误选项重新带出来。

## Implementation Steps

### 1. Store

对齐现有 `codex-store.ts` 风格，至少包含：

- profiles
- activeProfileId
- currentProfile
- isLoading
- error
- configStatus
- CRUD actions
- apply action
- launch action
- load from live config action

### 2. GUI 页面

建议整体交互风格对齐 `CodexConfigPage.tsx`：

- 左/上方 profile 选择
- 主区块编辑
- 顶部 apply/run 操作
- 状态卡片

### 3. TUI 页面

建议复用 Codex TUI 的信息架构：

- 列表页
- profile detail 页
- modal 输入/确认

### 4. CLI run/list

需要新增：

- `droidgear-tui run claude --list`
- `droidgear-tui run claude <selector>`

输出风格对齐当前：

- `run codex`
- `run droid`

## Acceptance Criteria

- GUI 可完整管理 Claude profiles
- GUI 可 apply / run / load from live config
- TUI 有同等 profile 管理能力
- `droidgear-tui run claude --list` 可列出可用 profile
- `droidgear-tui run claude <selector>` 可按 selector 启动
- 不出现 `xhigh` / `API Key` / auth inherit 误入口

## Tests

建议至少覆盖：

- store action tests
- GUI component tests
- launch 前自动保存当前 profile 的行为
- TUI CLI parser tests
- TUI list output / selector resolution tests
- run preview tests

## Dependencies

- 依赖 [task-x-claude-01-core-profile-storage-and-path-plumbing.md](./task-x-claude-01-core-profile-storage-and-path-plumbing.md)
- 依赖 [task-x-claude-02-apply-and-live-settings-merge.md](./task-x-claude-02-apply-and-live-settings-merge.md)
- 依赖 [task-x-claude-03-temp-run-runtime-contract-and-launcher-shim.md](./task-x-claude-03-temp-run-runtime-contract-and-launcher-shim.md)
