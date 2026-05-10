# Claude 03: Temporary Run Runtime Contract 与 Launcher Shim

## Overview

这是 Claude 线最关键的一段。

目标是把 Temporary Run 做成：

- 不污染 live settings
- 不泄漏 bearer token
- 能压过 inherited env 和 live `settings.env`
- 继续共享 `CLAUDE_CONFIG_DIR`

这一步必须直接参考 `codex-remote-feishu` 已经跑通的路径，不要重新发明一套半可靠机制。

## Goals

- 新增 `claude_runtime.rs`
- 定义 Claude temp run plan
- 定义 wrapper-private runtime settings contract
- 实现受管 env scrub
- 实现 `CLAUDE_ENV_FILE` copy-on-run
- 引入轻量 launcher shim
- 通过 shim 写出临时 `--settings` overlay
- 对需要清空的键写空字符串 tombstone

## Non-Goals

- 不在这一步完成完整 GUI/TUI 页面
- 不在这一步做 capability hint 高级配置
- 不在这一步接入 pinned alias `_SUPPORTED_CAPABILITIES`

## Contract Shape

runtime contract 至少要承载：

- `env.ANTHROPIC_BASE_URL`
- `env.ANTHROPIC_AUTH_TOKEN`
- `env.ANTHROPIC_MODEL`
- `env.ANTHROPIC_DEFAULT_HAIKU_MODEL`
- `env.CLAUDE_CODE_EFFORT_LEVEL`
- `env.CLAUDE_CODE_DISABLE_THINKING`
- `env.CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`
- `env.MAX_THINKING_TOKENS`
- top-level `alwaysThinkingEnabled`

其中：

- 需要清掉的 env key 用 `""` 作为 tombstone

## Launcher Choice

实现可以二选一，但 contract 不能变：

### 方案 A

- 新增独立 runner：
  - `src-tauri/crates/droidgear-tui/src/bin/droidgear-claude-runner.rs`

### 方案 B

- 在现有 `droidgear-tui` 里新增 hidden internal subcommand

推荐原则：

- 如果独立 bin 会明显增加打包复杂度，优先 hidden subcommand
- 但不论选哪种，launch planner 输出的 payload 结构必须保持稳定

## Suggested File Changes

- `src-tauri/crates/droidgear-core/src/claude_runtime.rs`
- `src-tauri/crates/droidgear-core/src/lib.rs`
- `src-tauri/src/commands/claude.rs`
- `src-tauri/src/utils/terminal_launch.rs`
  - 如需复用 `secret_env` / `support_dir`
- `src-tauri/crates/droidgear-tui/src/main.rs`
  - 若选择 hidden subcommand
- `src-tauri/crates/droidgear-tui/src/bin/droidgear-claude-runner.rs`
  - 若选择独立 runner
- `src-tauri/crates/droidgear-tui/src/tui/utils.rs`
  - 如需 CLI preview/list helper

## Planner Responsibilities

### 1. Build Temporary Run Plan

planner 需要产出：

- launch program
- launch args
- visible env
- secret env
- unset env
- runtime support dir
- wrapper-private payload
- warnings

### 2. Scrub inherited env

至少要 scrub：

- `ANTHROPIC_BASE_URL`
- `ANTHROPIC_AUTH_TOKEN`
- `ANTHROPIC_API_KEY`
- `ANTHROPIC_MODEL`
- `ANTHROPIC_DEFAULT_HAIKU_MODEL`
- `CLAUDE_CODE_EFFORT_LEVEL`
- `CLAUDE_CODE_DISABLE_THINKING`
- `MAX_THINKING_TOKENS`
- `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`
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

### 3. Private payload

private payload 只能给 runner / shim 自己看。

Claude child 真正启动前：

- runner 必须把 payload 从 child env 里清掉

### 4. `CLAUDE_ENV_FILE` handling

如果父环境有 `CLAUDE_ENV_FILE`：

1. 复制文件到 runtime dir
2. child 改指向副本

如果没有：

- 不伪造额外 env file

## Overlay Responsibilities

runner / shim 需要：

1. 读取 private payload
2. materialize Claude runtime settings JSON
3. 写到 runtime dir
4. 文件权限 `0600`
5. 给 Claude child 追加：
   - `--settings <path>`
6. 最后 `exec claude`

## Acceptance Criteria

- temp run 不修改 live `settings.json`
- temp run 能覆盖 inherited env
- temp run 能覆盖 live `settings.env` 冲突键
- bearer token 不出现在 terminal launch command line
- bearer token 不出现在 world-readable runtime file
- `CLAUDE_CONFIG_DIR` 继续共享
- `CLAUDE_ENV_FILE` 按 copy-on-run 工作

## Tests

建议至少覆盖：

- runtime contract JSON 生成
- tombstone key 会进入 overlay
- inherited env scrub 正确
- private payload 不继续传给 Claude child
- runner 生成的 launch args 不泄漏 secret
- overlay 文件权限测试
- `CLAUDE_ENV_FILE` 复制逻辑
- stale runtime overlay cleanup

## Dependencies

- 依赖 [task-x-claude-01-core-profile-storage-and-path-plumbing.md](./task-x-claude-01-core-profile-storage-and-path-plumbing.md)
- 依赖 [task-x-claude-02-apply-and-live-settings-merge.md](./task-x-claude-02-apply-and-live-settings-merge.md)
