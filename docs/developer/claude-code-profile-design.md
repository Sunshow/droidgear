# Claude Code Provider Profile Design

这份文档定义 Claude Code 多 profile、Apply、Temporary Run 的产品边界和实现约束。

这次要做的不是“Claude 官方模型选项管理器”，而是一个 provider-first 的配置系统：

- 配不同 provider / gateway
- 配不同 bearer token
- 配不同 model name
- 配 Claude Code 自己的 reasoning / thinking 默认行为

相关文档：

- See [temporary-tool-run-design.md](./temporary-tool-run-design.md)
- See [tauri-commands.md](./tauri-commands.md)
- See [tui-design.md](./tui-design.md)

官方参考：

- Claude Code settings: <https://code.claude.com/docs/en/settings>
- Claude Code model config: <https://code.claude.com/docs/en/model-config>
- Claude Code env vars: <https://code.claude.com/docs/en/env-vars>
- Claude Code CLI reference: <https://code.claude.com/docs/en/cli-reference>

## Product Goal

用户要的是一套和 Codex provider profile 类似的能力：

- 支持多个 Claude Code profile
- 每个 profile 的主轴是：
  - `base url`
  - `bearer token`
  - `model name`
- 支持 Apply 到 Claude 本地配置，让其成为默认值
- 支持 Temporary Run
- Temporary Run 只改我们明确托管的字段
- 其他 Claude Code 状态尽量继续共享 live 行为

本轮产品明确收敛为：

- 只支持 Anthropic-compatible 路径
- 不支持 `API Key`
- 不支持认证 `inherit` 模式切换
- 不支持 Bedrock / Vertex / Foundry 专项配置 UI

## Official Findings That Matter

### 1. Provider 配置的主入口确实是 env

Claude Code 官方直接支持这些和本需求强相关的变量：

- `ANTHROPIC_BASE_URL`
- `ANTHROPIC_AUTH_TOKEN`
- `ANTHROPIC_MODEL`
- `ANTHROPIC_DEFAULT_HAIKU_MODEL`

这意味着：

- `base url` 和 `bearer token` 本质上是 env-driven
- `small model` 也是 env-driven
- 主模型既可以通过 `ANTHROPIC_MODEL`，也可以通过 settings 的顶层 `model`

单看官方文档，会出现两种都“能用”的写法：

- provider/profile 语义走 `settings.env`
- 主模型和 effort 走 top-level `model` / `effortLevel`

但结合 `codex-remote-feishu` 已经跑通的 Claude temp run 路径，DroidGear 这轮应该收敛成更强的一致性约束：

- provider、主模型、small model、reasoning 全部走受管 `env` contract
- 只把 `alwaysThinkingEnabled` 留在 top-level

原因很直接：

- Apply 和 Temporary Run 可以共用同一套 managed keys
- launcher 可以统一做 env scrub + overlay tombstone
- 不需要同时维护 top-level `model` / `effortLevel` 和 `env` 两套优先级语义

### 2. `--settings` 是 session overlay，不是完整隔离层

官方 `claude --settings <file-or-json>` 的语义是：

- 只覆盖本次 session
- 只覆盖你明确提供的 key
- 未提供的 key 继续继承其他 settings source

这条很重要，因为它说明：

- Temporary Run 可以做“最小 overlay”
- 但要想可靠覆盖 lower-layer 配置，不能只靠“省略未设置字段”

`codex-remote-feishu` 的现成做法已经证明，正确路径不是“只写一个 overlay 文件”，而是三层一起做：

1. 父进程先 scrub 掉受管 env
2. daemon / launcher 冻结 wrapper-private runtime settings contract
3. wrapper / shim 再把 contract 写成临时 `--settings` overlay，并对需要清空的 key 写 tombstone

对 `CLAUDE_CONFIG_DIR` 还要再补一条约束：

- 只透传显式存在的 config-dir override
- 不要把默认 `~/.claude` 再包装成一个新的 `CLAUDE_CONFIG_DIR` export

所以这条在 DroidGear 里不应该继续停留在“要不要验证”的讨论，而应该直接采纳同类机制。

### 3. `CLAUDE_CONFIG_DIR` 影响的是整套 Claude 状态

官方说明 `CLAUDE_CONFIG_DIR` 会一起切走：

- settings
- credentials
- session history
- plugins

因此：

- 不能轻率把 profile 直接做成另一套独立 config dir
- 否则 resume / history / plugin 状态都会分叉

这条仍然支持“共享 live Claude 状态”的总体方向，但 temp run 里的正确实现不应该是“总是导出 `CLAUDE_CONFIG_DIR`”。

更稳妥的约束应该是：

- 如果用户显式配置了 Claude config path override，temp run 继续透传该 override
- 如果没有显式 override，temp run 默认回到手工 `claude --settings <overlay>` 的行为，不额外发明一个默认 `CLAUDE_CONFIG_DIR`

### 4. `CLAUDE_ENV_FILE` 也是 runtime 状态的一部分

官方说明 `CLAUDE_ENV_FILE`：

- 会在 Bash tool / hook 之前被 source
- 会被 `SessionStart` / `Setup` / `CwdChanged` / `FileChanged` hooks 追加内容

这意味着 temp run 不只有 settings 和父进程 env 两层：

- 还存在一个会被持续修改的 runtime env file

如果直接复用共享 `CLAUDE_ENV_FILE`：

- temp run 可能继承旧会话残留 env
- temp run 内 hook 也会反向污染外部会话

### 5. Claude 的 reasoning effort 不是统一能力面

官方当前文档明确写的是：

- Opus 4.7：`low / medium / high / xhigh / max`
- Opus 4.6、Sonnet 4.6：`low / medium / high / max`

同时官方还写明：

- `settings.effortLevel` 只接受 `low / medium / high / xhigh`
- `--effort` 支持 `low / medium / high / xhigh / max`
- `CLAUDE_CODE_EFFORT_LEVEL` 支持 `low / medium / high / xhigh / max / auto`
- `max` 默认是 session-only，除非通过 `CLAUDE_CODE_EFFORT_LEVEL`

因此这轮产品里原先把 `xhigh` 当成通用档位是错误的。

正确理解应该是：

- `xhigh` 只是 Opus 4.7 的特例能力
- 它不是 Claude 系列的稳定共通档位
- provider-first profile 不应该默认把它当通用选项暴露

### 6. Thinking 不只是一个 `alwaysThinkingEnabled`

官方 thinking 相关控制面至少有这几层：

- `alwaysThinkingEnabled`
  - 默认是否开启 thinking
- `showThinkingSummaries`
  - 只控制交互模式里 summary 的展示，不改变 thinking 是否发生
- `CLAUDE_CODE_DISABLE_THINKING=1`
  - 强制关闭 thinking，优先级很高
- `MAX_THINKING_TOKENS`
  - `0` 时直接禁用 thinking
  - 正数预算只在 fixed-budget 模式下起作用
- `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING=1`
  - 只对 Opus 4.6 / Sonnet 4.6 生效
  - 会把 thinking 从 adaptive 模式切回 fixed-budget 模式

官方还明确说明：

- Opus 4.7 永远使用 adaptive reasoning
- `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING` 对 Opus 4.7 无效
- 在支持 adaptive reasoning 的模型上，effort 才是控制 thinking 深度的主轴

因此 DroidGear 这里不能再把“thinking mode”理解成单个布尔开关。

### 7. 自定义 model name 会影响 effort / thinking 是否可用

这是本轮最容易被忽略，但影响很大的点。

官方明确说明：

- Claude Code 会通过 model ID pattern 判断这个模型是否支持 `effort`、`thinking`、`adaptive_thinking`
- Bedrock ARN、自定义 deployment name、网关自定义 model ID 往往匹配不到这些内建规则
- 一旦匹配不到，Claude Code 会直接把相关能力关掉

官方给出的补救方式是：

- 对 pinned alias 或 custom model option 设置 `_SUPPORTED_CAPABILITIES`

但官方也明确写了：

- 直接通过 `ANTHROPIC_MODEL` 或 `--model` 提供的值会按原样传给 provider
- `modelOverrides` 不会改写这些直接传入的值

这意味着：

- `model name` 做自由输入是对的
- 但对“任意自定义 model name + effort/thinking 一定可用”不能做过度承诺

v1 必须把这条写成显式产品边界。

## Revised V1 Product Surface

### Profile shape

```ts
type ClaudeReasoningEffort = 'low' | 'medium' | 'high' | 'max'
type ClaudeThinkingMode = 'inherit' | 'on' | 'off'

interface ClaudeCodeProfile {
  id: string
  name: string
  description?: string | null

  baseUrl?: string | null
  bearerToken?: string | null
  model?: string | null

  smallModelUsesMainModel: boolean
  smallModel?: string | null

  reasoningEffort?: ClaudeReasoningEffort | null
  thinkingMode: ClaudeThinkingMode

  createdAt: string
  updatedAt: string
}
```

语义：

- `baseUrl = null`：不管理 endpoint
- `bearerToken = null`：不提供 bearer token
- `model = null`：不管理主模型默认值
- `smallModelUsesMainModel = true`：small model 直接跟随主 model 名字
- `reasoningEffort = null`：不管理 reasoning effort
- `thinkingMode = inherit`：不管理 thinking 默认行为

### Why `xhigh` is removed

文档必须明确修正：

- `xhigh` 不再进入 DroidGear 的 Claude profile UI

原因不是“官方完全没有这个值”，而是：

- 官方只把它给 Opus 4.7
- Sonnet 4.6 和 Opus 4.6 都没有
- 我们这里做的是“provider-first + 自由 model name”
- 不是“官方 Opus 4.7 专用 preset 配置器”

所以 v1 暴露的 reasoning 选项应收敛为：

- `inherit`
- `low`
- `medium`
- `high`
- `max`

如果未来真的要支持：

- “当 capability hint 明确是 Opus 4.7 时再解锁 `xhigh`”

那应该在后续高级配置里做，不应该混进这轮通用面板。

### What v1 should include

| DroidGear field           | Claude 对应项                                                    | 说明                                     |
| ------------------------- | ---------------------------------------------------------------- | ---------------------------------------- |
| `name` / `description`    | DroidGear metadata                                               | 本地 profile 元数据                      |
| `baseUrl`                 | `env.ANTHROPIC_BASE_URL`                                         | 自由输入，可空                           |
| `bearerToken`             | `env.ANTHROPIC_AUTH_TOKEN`                                       | secret input，可空                       |
| `model`                   | `env.ANTHROPIC_MODEL`                                            | 自由输入，可空                           |
| `smallModel`              | `env.ANTHROPIC_DEFAULT_HAIKU_MODEL`                              | 自由输入，可空                           |
| `smallModelUsesMainModel` | behavior flag                                                    | 勾上后 small model 直接使用主 model 名字 |
| `reasoningEffort`         | `env.CLAUDE_CODE_EFFORT_LEVEL`                                   | UI 只暴露 `low / medium / high / max`    |
| `thinkingMode`            | `alwaysThinkingEnabled` + `CLAUDE_CODE_DISABLE_THINKING` hygiene | 三态 `inherit / on / off`                |

### What v1 should explicitly exclude

本轮不收：

- `API Key`
- 认证 `inherit` 模式
- 自定义 header 认证
- `apiKeyHelper`
- `showThinkingSummaries`
- `MAX_THINKING_TOKENS` 的正数预算配置
- `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING` 独立开关
- pinned alias `_SUPPORTED_CAPABILITIES` UI
- `availableModels`
- `modelOverrides`
- Bedrock / Vertex / Foundry / Foundry API key 专项配置

这里最重要的产品决定是：

- v1 只做“Anthropic-compatible provider profile”
- 不做“Claude 所有 provider 方言的统一控制台”

## Small Model Constraint

Claude 官方当前只提供：

- 全局 `base url`
- 全局认证
- 独立 `ANTHROPIC_DEFAULT_HAIKU_MODEL`

所以 small model 的正确产品边界必须写死：

- 可以有独立 small model name
- 不能有独立 base URL
- 不能有独立 bearer token

因此：

- `smallModelUsesMainModel = true`
  - 若主 `model` 非空，则 `ANTHROPIC_DEFAULT_HAIKU_MODEL = model`
- `smallModelUsesMainModel = false`
  - 允许填写 `smallModel`
  - 但仍与主模型共享同一套 provider / auth

这不是取舍问题，而是官方能力上限。

## Thinking Design, Revised

### v1 的 thinking 只管理默认开关，不管理 budget 策略

基于官方能力面，v1 应把 thinking 收敛成：

- `inherit`
- `on`
- `off`

它只表达：

- 是否默认开启 thinking
- 是否强制关闭 thinking

它不表达：

- fixed-budget token 数值
- adaptive / fixed 模式切换
- thinking summary 是否展示

### Why budget / adaptive stay out of v1

原因有三个：

1. `MAX_THINKING_TOKENS` 的正数预算只在 fixed-budget 模式有意义
2. `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING` 只对 Opus 4.6 / Sonnet 4.6 生效，对 Opus 4.7 无效
3. provider-first profile 如果同时暴露 effort、adaptive、budget，会很快把 UI 变成模型版本专用面板

因此 v1 的 clean contract 应该是：

- `thinkingMode = on`
  - 使用 Claude 默认的 thinking 路径
  - 不再额外托管 fixed-budget 行为
- `thinkingMode = off`
  - 强制禁用 thinking

### Concrete semantics

- `thinkingMode = inherit`
  - 不写 `alwaysThinkingEnabled`
  - 不写 `CLAUDE_CODE_DISABLE_THINKING`
  - 不托管 thinking 相关 env
- `thinkingMode = on`
  - `alwaysThinkingEnabled = true`
  - 清理 `CLAUDE_CODE_DISABLE_THINKING`
  - 清理 `MAX_THINKING_TOKENS`
  - 清理 `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`
- `thinkingMode = off`
  - `alwaysThinkingEnabled = false`
  - 设置 `CLAUDE_CODE_DISABLE_THINKING = 1`
  - 清理 `MAX_THINKING_TOKENS`
  - 清理 `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`

这里“清理 budget / adaptive key”是有意为之。

原因是：

- 只要继续继承这些 key，`thinkingMode = on` 就不能保证得到可预测结果
- 既然 v1 不暴露 fixed-budget / adaptive 控制，就应该把它们从受管 thinking 语义里排除掉

换句话说：

- DroidGear v1 一旦管理 thinking，就默认回到 Claude 官方默认的 adaptive thinking 路径

### `showThinkingSummaries` 的处理

`showThinkingSummaries` 只影响展示，不影响 thinking 是否发生。

所以 v1 的正确做法是：

- 不托管它
- Temporary Run / Apply 都继续继承用户当前展示偏好

## Reasoning Design, Revised

### Reasoning 只暴露稳定的四档

UI 只暴露：

- `inherit`
- `low`
- `medium`
- `high`
- `max`

不暴露：

- `xhigh`

### Apply semantics for effort

`reasoningEffort = null`

- 删除 `env.CLAUDE_CODE_EFFORT_LEVEL`

`reasoningEffort = low | medium | high`

- 写 `env.CLAUDE_CODE_EFFORT_LEVEL`
- `low / medium` 清理 `env.CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`
- `high` 写 `env.CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING = 1`

`reasoningEffort = max`

- 写 `env.CLAUDE_CODE_EFFORT_LEVEL = max`
- 写 `env.CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING = 1`

这样做的原因是：

- 这轮已经决定让 reasoning 和 provider/model 一起走统一 env contract
- `max`、`high`、`medium`、`low` 都能走一套一致语义
- 可以直接复用 `codex-remote-feishu` 里已经跑通的 runtime settings contract 设计

### Temporary Run semantics for effort

Temporary Run 也不再单独走 `--effort`。

它应该和 provider/model 一起进入同一个受管 runtime env contract：

- `env.CLAUDE_CODE_EFFORT_LEVEL`
- `env.CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`
- 必要时清理 `env.CLAUDE_CODE_DISABLE_THINKING`

这样做的好处是：

- Apply 和 Temporary Run 的 reasoning 语义完全一致
- 不需要同时维护 “profile 默认值” 和 “run-time flag” 两条路线
- 可以在 wrapper/shim 层一次性解决 inherited env 和 lower-layer settings 污染

## Capability Detection Boundary

这条边界必须写进产品文档，否则 UI 会误导用户。

当 profile 使用的是：

- 官方可识别的 Claude model ID

那么：

- effort / thinking 通常能正常工作

当 profile 使用的是：

- 网关自定义 model name
- provider-specific deployment name
- 不符合 Claude 内建 pattern 的 model ID

那么：

- Claude Code 可能直接判定它“不支持 effort / thinking”

因此 v1 必须明确：

- `reasoningEffort` 和 `thinkingMode` 是“best-effort on supported models”
- 对 opaque custom model ID，不承诺 Claude 一定启用这些能力

当前实现 contract 也应该写死：

- 当 model 非空且看起来不是官方 `claude-*` 风格 ID 时，GUI / TUI 必须显式提示 capability boundary
- launch 前应先在 login shell 语境探测 `claude --version`，避免“终端打开成功但 Claude 实际没装”被误判成启动成功，也避免 GUI 进程自身 `PATH` 与终端 shell 不一致时误报
- `CLAUDE_ENV_FILE` 复制失败应作为 warning surfaced，而不是静默降级

如果后续要把这件事做干净，应该单独设计一层高级能力提示，例如：

- `capabilityPreset = auto | sonnet-4.6 | opus-4.6 | opus-4.7`

然后转为走：

- pinned alias env
- `_SUPPORTED_CAPABILITIES`

但这不应该混进本轮最小产品面。

## Apply Semantics

### Target

Apply 的语义固定为：

- 写入当前有效的 Claude user config

目标路径：

- `<CLAUDE_CONFIG_DIR>/settings.json`
- 若未设置 `CLAUDE_CONFIG_DIR`，回退 `~/.claude/settings.json`

### Managed keys

Apply 只改这些 key：

- top-level `alwaysThinkingEnabled`
- `env.ANTHROPIC_BASE_URL`
- `env.ANTHROPIC_AUTH_TOKEN`
- `env.ANTHROPIC_MODEL`
- `env.ANTHROPIC_DEFAULT_HAIKU_MODEL`
- `env.CLAUDE_CODE_EFFORT_LEVEL`
- `env.CLAUDE_CODE_DISABLE_THINKING`
- `env.MAX_THINKING_TOKENS`
- `env.CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`

Apply 还需要清理这些冲突键：

- `env.ANTHROPIC_API_KEY`
- 所有替代 provider 旁路 env
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

原因：

- 本轮认证只支持 bearer token
- provider / model / reasoning 这轮统一走受管 env contract
- 不能让旧 env 抢走 profile 语义

同时这轮明确不主动清理这些 top-level 用户默认值：

- `model`
- `effortLevel`

原因是：

- v1 已经决定由 `env` 承载 DroidGear 的受管 provider contract
- top-level `model` / `effortLevel` 继续保留为用户自己的 live 默认层
- 这样对现有 Claude 配置的侵入更小，也更符合“只动受管字段”的原则

### Security caveat

如果用户选择 Apply，且 profile 配了 bearer token：

- token 会以明文落在 Claude 的 `settings.json` 里

这不是 DroidGear 独有问题，而是 Claude 官方当前配置模型的直接结果。

因此产品上必须明确：

- `Apply` = 持久写入 live config
- 不想把 token 持久落盘，就用 `Temporary Run`

## Temporary Run Semantics

Claude 的 temp run 目标不是强隔离容器，而是：

- 不预先污染 live settings
- 启动的 Claude 实例带着选中 profile 的 provider / auth / model 运行
- 尽量继续共享 session / history / plugins / trust

### Preferred runtime contract

这里直接采用 `codex-remote-feishu` 那条已经成型的思路：

1. 共享 live Claude config state，但不默认硬塞 `CLAUDE_CONFIG_DIR`
2. launcher 先 scrub 受管 Claude env
3. 后端冻结 wrapper-private runtime settings contract
4. 轻量 wrapper / shim 把 contract 写成 `--settings <temp-file>` overlay
5. `CLAUDE_ENV_FILE` copy-on-run

在 DroidGear 里，这里进一步收敛成一个明确实现约束：

- planner 不能预写 Claude overlay 文件
- planner 不能预先复制 `CLAUDE_ENV_FILE`
- planner 只负责构造 launch plan 和 wrapper-private payload
- 真正的 overlay materialize、`CLAUDE_ENV_FILE` copy、payload scrub 必须发生在 internal launcher 里

internal launcher 可以是：

- Tauri 主二进制的 hidden internal subcommand
- TUI 二进制的 hidden internal subcommand

但不允许回退成“主流程直接拼 `claude --settings <path>` 并提前落盘”的实现。

这里的关键点不是“只有一个 overlay 文件”，而是：

- 父环境先去脏
- child settings 再显式盖住 live `settings.env`
- 需要清空的 key 用 tombstone 明确表达

### Managed runtime contract

参考 `codex-remote-feishu`，DroidGear 这轮也应该引入一份 wrapper-private Claude runtime contract。

它至少需要承载：

- `env.ANTHROPIC_BASE_URL`
- `env.ANTHROPIC_AUTH_TOKEN`
- `env.ANTHROPIC_MODEL`
- `env.ANTHROPIC_DEFAULT_HAIKU_MODEL`
- `env.CLAUDE_CODE_EFFORT_LEVEL`
- `env.CLAUDE_CODE_DISABLE_THINKING`
- `env.CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`
- `env.MAX_THINKING_TOKENS`
- top-level `alwaysThinkingEnabled`
- `runtimeDir`
- `liveConfigDir`
- optional `inheritedClaudeEnvFileSource`

其中需要“清掉”的 key 不能简单省略，而要显式写 tombstone。

在参考实现里，这个 tombstone 语义就是：

- `env.<KEY> = ""`

对 DroidGear 来说，推荐同样采用这套 contract，而不是重新发明另一种删除协议。

### Overlay should own

overlay 只写这份受管 runtime contract：

- 受管 `env` 键
- 必要的 `alwaysThinkingEnabled`

并且：

- 不复制整份 live settings
- 不改无关 Claude config
- overlay 文件权限必须是 `0600`
- overlay 文件名应放在 runtime dir 下，并做 stale cleanup

并且这份 overlay：

- 只能由 internal launcher 在真正 exec `claude` 前生成
- preview 可以展示它的 JSON 内容，但 preview 不得把它物化到磁盘

### Child env hygiene

如果 temp run 不做 env hygiene，父环境里的旧值会覆盖 profile。

至少必须处理：

- `ANTHROPIC_BASE_URL`
- `ANTHROPIC_AUTH_TOKEN`
- `ANTHROPIC_API_KEY`
- `ANTHROPIC_MODEL`
- `ANTHROPIC_DEFAULT_HAIKU_MODEL`
- `CLAUDE_CODE_EFFORT_LEVEL`
- `CLAUDE_CODE_DISABLE_THINKING`
- `MAX_THINKING_TOKENS`
- `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`

另外还应该清理所有会把会话带到别的 provider 路径上的 inherited env：

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

否则用户选的是 Claude profile，实际请求仍可能被继承环境带偏。

这里要强调一点：

- env scrub 只负责处理“进程继承环境”
- 它不能解决 live `settings.json` / project settings 里的同名 `env`

所以 temp run 必须同时拥有：

- 父环境 scrub
- child overlay tombstone

缺任何一层都不够。

### `CLAUDE_ENV_FILE` copy-on-run

如果父环境里已经有：

- `CLAUDE_ENV_FILE=/path/to/file`

temp run 不应直接复用，而应：

1. 复制现有文件内容到 runtime dir
2. 把 child env 的 `CLAUDE_ENV_FILE` 指向副本

这里的“父环境”在 DroidGear 里不是指最终 `claude` child 的进程环境，而是指 internal launcher 收到的那份冻结 payload 里的 source path。

也就是说：

- planner 读取当前进程里的 `CLAUDE_ENV_FILE`
- 把归一化后的 source path 冻结进 wrapper-private payload
- internal launcher 在运行时读取 source path，复制文件，并把 child env 指向 runtime 副本

这样可以同时满足：

- 尽量继承当前 runtime env 基线
- temp run 内 hook 对 env file 的追加不会污染外部会话

### How lower-layer override is guaranteed

这条现在不再作为“未知风险”保留，而是直接定义成实现 contract：

1. GUI / TUI / Rust launch planner 先从启动环境里 scrub 掉受管 Claude env
2. planner 把最终受管值冻结进一个 wrapper-private payload
3. wrapper / shim 在真正 exec `claude` 前，把 payload 写成临时 `--settings` overlay
4. overlay 里对需要清掉的 key 写空字符串 tombstone
5. wrapper / shim 自己消费完 private payload 后，要把该 private env 从 child env 里移除
6. wrapper / shim 自己完成 `CLAUDE_ENV_FILE` copy-on-run，并把 child env 的 `CLAUDE_ENV_FILE` 指向 runtime 副本

这样分别覆盖三类污染源：

- 继承 shell env
- live `settings.env`
- wrapper 自己的中间态私有 payload

这也是为什么 DroidGear 这轮不应该继续尝试“直接在 terminal command line 上拼一堆 Claude env 再启动”。

更干净的路径是：

- 增加一个轻量 Claude temp-run launcher shim
- 它只负责 materialize runtime contract，然后 `exec claude`

在本仓库里，推荐把这个 shim 做成 hidden internal subcommand，并由 GUI / TUI 各自用当前可执行文件去启动自己的 internal launcher：

- GUI temporary run 启动当前 Tauri app binary 的 internal launcher
- TUI temporary run 启动当前 `droidgear-tui` binary 的 internal launcher

两边都调用同一套 `droidgear-core` Claude launcher helper，避免再出现两份不一致实现。

这样既能避免 secret 出现在 terminal launch command line，也能把 override 逻辑收口到一处。

## Required Repo Changes Before Implementation

### 1. Add Claude config resolution

至少支持：

- 默认 `~/.claude`
- 自定义 `CLAUDE_CONFIG_DIR`
- GUI / TUI / Rust core 共用同一套解析

### 2. Add Claude core/runtime module

建议新增：

- `src-tauri/crates/droidgear-core/src/claude.rs`
- `src-tauri/crates/droidgear-core/src/claude_runtime.rs`
- `src-tauri/crates/droidgear-tui/src/main.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/commands/claude.rs`
- `src/store/claude-store.ts`
- `src/components/claude/*`

这里的 launcher shim 不是可选优化，而是推荐主路径。

原因：

- 需要先消费 wrapper-private runtime contract
- 需要在本地写 `0600` overlay 文件
- 需要 copy `CLAUDE_ENV_FILE`
- 需要避免把 bearer token 暴露到 terminal launch command line

### 3. Define a strict managed-key contract

必须先固定：

- Apply 管哪些 key
- Temp run 管哪些 key
- 哪些 inherited env 必须 clear
- 哪些 live `settings.env` key 需要写空字符串 tombstone
- `CLAUDE_ENV_FILE` 如何复制
- 哪些值可以进命令行，哪些只能进 runtime file
- wrapper-private payload 的 env 名称、格式和清理时机

### 4. Add a real temp-run correctness test matrix

至少覆盖：

- inherited provider env 不会覆盖 profile
- inherited alternative-provider env 不会把请求带偏
- live `settings.env` 里的冲突 key 会被 overlay tombstone 盖掉
- 默认 temp run 路径与手工 `claude --settings <overlay>` 一致，不额外导出默认 `CLAUDE_CONFIG_DIR`
- 若用户显式配置了 Claude config path override，temp run 继续共享该 override
- `CLAUDE_ENV_FILE` 被复制而不是复用
- temp overlay 不把 bearer token 泄漏到命令行
- wrapper-private runtime payload 不会继续传给 Claude child
- Apply 只改受管 key，不覆盖无关 settings
- `smallModelUsesMainModel=true` 时 Haiku model 跟随主 model
- `reasoningEffort=max` 时 Apply / Temporary Run 都按预期工作
- `thinkingMode=on/off` 时 `MAX_THINKING_TOKENS` / `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING` 不会残留出错
- custom opaque model ID 下，effort / thinking capability 缺失时 UI 或日志能正确告知用户

## Final Recommendation

修正后的结论是：

- Claude profile 仍然应该做成 provider-first
- 主轴仍然是 `base url + bearer token + model name`
- 但 reasoning / thinking 的设计必须更保守、更贴官方真实能力面

具体建议：

- `reasoningEffort`
  - v1 只暴露 `low / medium / high / max`
  - 删除 `xhigh`
- `thinkingMode`
  - 继续保留 `inherit / on / off`
  - 但文档和实现都要明确：v1 不支持 fixed-budget / adaptive 专项配置
- `Apply`
  - provider / model / reasoning 统一走受管 `env` contract
  - 只把 `alwaysThinkingEnabled` 留在 top-level
- `Temporary Run`
  - 默认不额外导出 `CLAUDE_CONFIG_DIR`
  - 若存在显式 Claude config path override，则继续共享该 override
  - 按 `codex-remote-feishu` 同类机制实现：
    - 父环境 scrub
    - wrapper-private runtime contract
    - 临时 `--settings` overlay
    - 空字符串 tombstone
    - `CLAUDE_ENV_FILE` copy-on-run
  - 不再建议直接拼命令行 env 启动 Claude

同时必须写清楚两个产品边界：

1. Claude 原生 small model 只能单独配 model name，不能单独配 provider / auth
2. 对任意自定义 model ID，不承诺 Claude 一定启用 effort / thinking，除非未来补做 capability hint / pinned alias 方案

这版设计比之前更接近 Claude 官方真实语义，也更适合作为后续实现和 PR 讨论的基础。
