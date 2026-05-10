# Claude Code Profiles 实现拆分路线

## Overview

这份文档把 [Claude Code Provider Profile Design](../developer/claude-code-profile-design.md) 细化成可分段开工的执行路线。

目标不是一次性“全做完”，而是拆成几段可以独立合并、独立测试、独立回归的工作包。

## Frozen Decisions

这些约束在实现阶段视为已定，不再反复摇摆：

- 只支持 `Bearer Token`
- 不支持 `API Key`
- 不支持认证 `inherit` 模式切换
- `reasoningEffort` 只暴露：
  - `low`
  - `medium`
  - `high`
  - `max`
- 不暴露 `xhigh`
- provider / model / small model / reasoning 统一走受管 `env` contract
- 只把 `alwaysThinkingEnabled` 留在 top-level settings
- `Temporary Run` 固定共享 live `CLAUDE_CONFIG_DIR`
- `Temporary Run` 固定采用：
  - 父环境 scrub
  - wrapper-private runtime contract
  - 临时 `--settings` overlay
  - 空字符串 tombstone
  - `CLAUDE_ENV_FILE` copy-on-run
- 不引入 profile-scoped `CLAUDE_CONFIG_DIR`
- GUI 和 TUI 必须同轮接入

## Recommended Execution Order

### 1. 基座与路径

See [task-x-claude-01-core-profile-storage-and-path-plumbing.md](./task-x-claude-01-core-profile-storage-and-path-plumbing.md)

先把 Claude profile 的数据模型、存储、路径解析、Tauri 命令合同立起来。

### 2. Apply 语义

See [task-x-claude-02-apply-and-live-settings-merge.md](./task-x-claude-02-apply-and-live-settings-merge.md)

把“把 profile 持久写入 Claude live settings”的行为做干净，顺便补上 live config 读取与 merge 行为。

### 3. Temporary Run

See [task-x-claude-03-temp-run-runtime-contract-and-launcher-shim.md](./task-x-claude-03-temp-run-runtime-contract-and-launcher-shim.md)

这是这条线最容易做脏的部分。必须把 runtime contract、shim、overlay tombstone 一次做对。

### 4. GUI / TUI 接入

See [task-x-claude-04-gui-tui-profile-management-and-run-surface.md](./task-x-claude-04-gui-tui-profile-management-and-run-surface.md)

在前两段后再开放真实用户入口，避免先做 UI 再被底层合同反复打回。

### 5. Guardrails 与整体验证

See [task-x-claude-05-capability-guardrails-and-integration-hardening.md](./task-x-claude-05-capability-guardrails-and-integration-hardening.md)

最后收 shared helper、warning、characterization 测试和最终文档更新。

## PR Slice Recommendation

如果后续要往上游提 PR，建议按下面的合并粒度切：

1. `01 + 02`
   - Claude profile core + apply 语义
2. `03`
   - Claude temporary run runtime contract + launcher
3. `04 + 05`
   - GUI/TUI + guardrails + full test matrix

这样每个 PR 的风险边界更清楚：

- 第一组只碰存储与 live config
- 第二组只碰 launch/runtime
- 第三组才碰用户入口与交互

## Cross-Cutting Rules

### 不引入 synthetic live/inherit profile

不要像某些工具那样引入一个“继承当前脏环境”的内建 profile。

原因：

- Claude 这条线最需要避免的就是不透明继承
- synthetic inherit profile 会把 temp run 的正确性又带脏

如果需要便捷起步，应该提供：

- `create_default_claude_profile()`

但它产出的仍然是显式、可编辑、可持久化的普通 profile。

### 不复用错误的 Claude xhigh helper

仓库当前共享 helper 里已有把 Claude 统一当成支持 `xhigh` 的逻辑。

Claude profile 接入时不能直接复用这些旧判断，否则 UI 会重新暴露错误选项。

### 不把 secret 暴露到 terminal command line

Claude temp run 里的：

- `ANTHROPIC_AUTH_TOKEN`

只能进入：

- wrapper-private payload
- `0600` runtime settings overlay
- child env

不能直接展开到：

- macOS AppleScript command string
- Linux shell command line
- Windows terminal launch args

### 不依赖人工回归

这条线必须靠自动测试覆盖：

- profile CRUD
- apply merge
- temp run env scrub
- overlay tombstone
- launcher secret hygiene
- GUI/TUI surface
- CLI list/run selectors

## Exit Condition

整条 Claude 线完成的标准是：

- GUI 可创建、编辑、应用、运行 Claude profile
- TUI 有同等配置能力
- `droidgear-tui run claude --list`
- `droidgear-tui run claude <selector>`
- Apply 不乱改无关 settings
- Temporary Run 不污染 live settings，且能覆盖 inherited/live 冲突配置
- `npm run check:all` 全绿
