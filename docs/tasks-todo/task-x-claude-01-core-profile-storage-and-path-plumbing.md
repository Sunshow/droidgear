# Claude 01: Core Profile Storage 与 Path Plumbing

## Overview

先把 Claude profile 的基础结构做出来：

- core types
- profile CRUD
- active profile state
- Claude config path resolution
- Tauri command contract

这一步完成后，仓库应该已经有一条稳定的 Claude profile 数据链路，但还没有真正去改 live settings，也还没有 temp run。

## Goals

- 新增 `droidgear-core` 的 Claude 模块
- 定义 Claude profile 数据模型
- 支持：
  - list
  - get
  - save
  - delete
  - duplicate
  - create default profile
  - get/set active profile id
- 扩展 `paths.rs`，让 Claude 有正式的配置路径解析入口
- 在 Tauri command/bindings 层暴露 Claude profile CRUD 合同
- 在 Preferences 的 Paths 面板里加入 Claude config 路径

## Non-Goals

- 不在这一步实现 Apply
- 不在这一步实现 Temporary Run
- 不在这一步开放 GUI/TUI 的完整 Claude 页面
- 不在这一步实现 launcher shim

## Key Decisions

### 1. 不引入 synthetic inherit profile

这一步不要新增“继承当前 live Claude 环境”的内建 profile。

可选的是：

- `create_default_claude_profile()`

但它应返回一个普通、显式、可编辑的空白 profile，而不是暗含“继承当前 shell/env/settings”的特殊对象。

### 2. Claude 路径进入统一 `paths` 系统

不要只在 Claude 模块里临时读取 `CLAUDE_CONFIG_DIR`。

需要把 Claude 路径纳入仓库现有的：

- `ConfigPaths`
- `EffectivePaths`
- Preferences / Paths Pane

这样 GUI、TUI、core、runtime 才会共用同一套解析。

## Suggested File Changes

- `src-tauri/crates/droidgear-core/src/claude.rs`
- `src-tauri/crates/droidgear-core/src/lib.rs`
- `src-tauri/crates/droidgear-core/src/paths.rs`
- `src-tauri/src/commands/claude.rs`
- `src-tauri/src/main.rs`
- `src/components/preferences/panes/PathsPane.tsx`
- `locales/en.json`
- `locales/zh.json`
- `src/lib/tauri-bindings.ts`
  - 通过 `npm run rust:bindings` 生成

## Data Shape

v1 profile 结构应与设计文档一致：

```ts
interface ClaudeCodeProfile {
  id: string
  name: string
  description?: string | null

  baseUrl?: string | null
  bearerToken?: string | null
  model?: string | null

  smallModelUsesMainModel: boolean
  smallModel?: string | null

  reasoningEffort?: 'low' | 'medium' | 'high' | 'max' | null
  thinkingMode: 'inherit' | 'on' | 'off'

  createdAt: string
  updatedAt: string
}
```

## Storage Layout

建议对齐 Codex 的做法：

- `~/.droidgear/claude/profiles/<id>.json`
- `~/.droidgear/claude/active-profile.txt`

这样：

- core/TUI/desktop 都能共用
- 和现有工具 profile 存储习惯一致

## Path Resolution Requirements

需要新增 Claude 的 effective path 入口，至少覆盖：

- 默认：
  - `~/.claude`
- 用户 override：
  - DroidGear 自己保存的 Claude path override

不要试图在这一层直接依赖父 shell 的 `CLAUDE_CONFIG_DIR`。

原因：

- GUI 启动路径里不一定能可靠继承 shell env
- 仓库当前已有正式路径配置系统，应优先走这个系统

## Implementation Steps

### 1. 新增 core Claude 模块

- 定义 types
- 定义 path helpers
- 定义 CRUD
- 定义 active profile state helpers

### 2. 扩展 `paths.rs`

- `ConfigPaths` 增加 `claude`
- `EffectivePaths` 增加 `claude`
- 新增：
  - `default_claude_home_for_home()`
  - `get_claude_home_for_home()`

### 3. Tauri command 层接线

至少需要：

- `list_claude_profiles`
- `get_claude_profile`
- `save_claude_profile`
- `delete_claude_profile`
- `duplicate_claude_profile`
- `create_default_claude_profile`
- `get_active_claude_profile_id`
- `set_active_claude_profile_id`

### 4. Preferences Paths Pane 增加 Claude 行

和现有：

- Codex
- OpenClaw
- Hermes

保持同样的视觉与交互结构。

## Acceptance Criteria

- Claude profile 可完整 CRUD
- 可持久化 active profile id
- Claude path 可在 Preferences 中查看、修改、重置
- `paths.rs` 能返回 Claude 的 effective path
- Tauri bindings 已生成，前端可调用 Claude CRUD 命令

## Tests

建议至少覆盖：

- Claude profile save/list/get roundtrip
- duplicate / delete / active profile id roundtrip
- profile id normalization / validation
- 空路径与默认路径行为
- `ConfigPaths.claude` 的 load/save/effective path 测试

## Dependencies

无前置依赖。

这是后续 Apply、Temporary Run、GUI/TUI 的共同前置。
