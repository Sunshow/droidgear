# Claude 05: Capability Guardrails 与 Integration Hardening

## Overview

最后一段专门收“容易漏但会长期制造误导”的东西：

- shared helper 修正
- opaque custom model warning
- characterization tests
- 失败路径与文案校正

这一步完成后，Claude 线才算真正可交付，而不是“主流程能跑，但边角到处漏”。

## Goals

- 修正共享 reasoning helper 与测试里的 Claude `xhigh` 假设
- 为 opaque custom model ID 增加能力边界提示
- 补齐 Claude apply / temp run / GUI / TUI 的 characterization coverage
- 补齐错误文案与 fallback 路径
- 更新用户文档

## Non-Goals

- 不做高级 capability preset
- 不做 `_SUPPORTED_CAPABILITIES` UI
- 不做 Bedrock / Vertex / Foundry 专项功能支持

## Suggested File Changes

- `src/lib/utils.ts`
- `src/lib/utils.test.ts`
- `src/components/claude/*.tsx`
  - warning / help text
- `src-tauri/crates/droidgear-core/tests/characterization.rs`
- `src-tauri/crates/droidgear-core/src/claude.rs`
- `src-tauri/crates/droidgear-core/src/claude_runtime.rs`
- `src-tauri/crates/droidgear-tui/src/tui/tests.rs`
- `docs/developer/claude-code-profile-design.md`
- `docs/userguide/`
  - 如需补充用户可见说明

## Hardening Topics

### 1. Shared helper cleanup

当前共享 helper 里至少已有这些潜在冲突：

- 把所有 Claude 模型都视为支持 `xhigh`
- 对 Claude thinking / effort 能力做过度泛化

如果 Claude 页面直接或间接用到这些 helper，结果一定会偏。

需要明确：

- 是把 helper 修正成真实语义
- 还是把 Claude 页面彻底与这些 helper 脱钩

### 2. Opaque model warning

当用户填的是：

- 网关自定义 model id
- provider-specific deployment name

需要明确提示：

- Claude Code 可能识别不出 `effort` / `thinking` 能力
- DroidGear 不保证这些能力一定生效

这类提示可以是：

- inline warning
- tooltip
- status text

但不能完全 silent。

### 3. Error-path correctness

至少要校正这些场景的反馈：

- Claude CLI 缺失
- launcher shim 执行失败
- live `settings.json` parse error
- temp overlay 写入失败
- `CLAUDE_ENV_FILE` 复制失败

### 4. Characterization coverage

需要有可以长期锁语义的 characterization 测试，避免以后改临近模块时把 Claude 搞脏。

## Acceptance Criteria

- Claude 页面不会再通过共享 helper 暴露 `xhigh`
- 自定义 opaque model id 有明确能力边界提示
- 关键失败路径都有明确错误反馈
- characterization 测试能锁住 apply / temp run / launch surface 语义

## Tests

建议至少覆盖：

- `supportsXhighEffort()` 或等价 helper 的 Claude 语义修正
- Claude 页面不会渲染 `xhigh`
- opaque model warning 渲染测试
- malformed live settings 的 GUI/TUI 错误反馈
- temp run overlay tombstone 的 characterization
- CLI list/run 失败路径

## Dependencies

- 依赖 [task-x-claude-01-core-profile-storage-and-path-plumbing.md](./task-x-claude-01-core-profile-storage-and-path-plumbing.md)
- 依赖 [task-x-claude-02-apply-and-live-settings-merge.md](./task-x-claude-02-apply-and-live-settings-merge.md)
- 依赖 [task-x-claude-03-temp-run-runtime-contract-and-launcher-shim.md](./task-x-claude-03-temp-run-runtime-contract-and-launcher-shim.md)
- 依赖 [task-x-claude-04-gui-tui-profile-management-and-run-surface.md](./task-x-claude-04-gui-tui-profile-management-and-run-surface.md)
