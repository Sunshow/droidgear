# DSH 配置面与 Dsh 页可视化设计

这份文档回答一个问题：除了现有的「模型服务商 / API Key」，还能从 DeepSeek Harness（下称 DSH）的本地配置里扒出什么，做成 Dsh 页的可视化配置。

基线版本：**DSH 0.2.1-rc.2**（与已安装的 `DeepSeek Harness.app` 一致，源码 git tag `dsh-v0.2.1-rc.2`）。DSH 的 rc 之间会删改配置键，本文所有结论都带源码出处，升级 DSH 后需要复核。

相关文档：

- See [tauri-commands.md](./tauri-commands.md)
- See [data-persistence.md](./data-persistence.md)
- See [tui-design.md](./tui-design.md)

## TL;DR

1. **写回目标只有一个**：`~/.dsh/profiles/<profile>/cordis.patch.yml`。DSH 自己的设置 UI 也是写它（经 `ctx.configEditor`）。
2. **`settings.yaml` 已废**：0.2.1 只在启动时把它一次性导入 profile patch，然后改名 `settings.yaml.imported`。DroidGear 现有的「legacy 判断」结论正确，但**无 profile 时回退写 `settings.yaml` 的代码路径应当删除**。
3. **能扒的很多**：desktop profile 有 170 个活动条目，其中 84 个声明配置、共 353 个字段（A 档 47 / B 档 50 / C 档 256）。DSH 自带 UI 只覆盖其中约 40 个 volatile 字段 + 4 张插件卡，其余字段只能手改 YAML——这正是 Dsh 页的价值空间。
4. **DSH 没有自动表单**：0.2.1 的设置页全是手写 React，`autoGenerate` 只是元数据、没有客户端消费。schema 只存在于运行时 `settings.describe()`，**文件里没有**。所以 DroidGear 不能「反射」schema，只能内置一份从 DSH 源码生成的 schema 快照。
5. **不要调运行中的桌面实例 API**：`/api/settings/*` 用 launch-token → 签名 cookie 鉴权，桌面版 token 不落盘，外部进程无受支持凭据；自己拉 `dsh web` sidecar 抓 token 可行，但拿到的是 owner 全权限。
6. **推荐路线**：直读/直写 profile patch + 镜像 DSH 的写入协议（同一把 `<profile>/package.json.lock`、随机临时文件、YAML Document 保注释/`!!js`、整段 config 替换、删除 config = 恢复默认、空文件必须写 `[]`）。

## 1. DSH 的配置模型

### 1.1 层叠（低 → 高）

```
profiles/<name>/cordis.yml            生成的空根，每次启动被重写为 []，不是配置
  ↓
bundle 层                               package.json dsh.profile.bundles 顺序；每个 bundle 的
                                        dsh.bundle.patch 文件（string | string[]）按声明顺序
  ↓
profiles/<name>/cordis.patch.yml       用户层，DSH 设置页写回目标 ← DroidGear 只写这里
  ↓
~/.dsh/cordis.patch.yml                home 层，高于 profile 层；只读
  ↓
--patch <file> ...                     启动覆盖层；desktop 宿主固定不传
  ↓
telemetry hard-disable patch            DSH_TELEMETRY_DISABLED 非空时自动追加
```

所有层摊平后一次 `applyEntryPatches`，同一 `id` 后出现者胜（`packages/boot/app-boot/src/profile.ts:816-823`、`profile-context.ts:74-84`）。

desktop profile 的 bundle 组合（`~/.dsh/profiles/desktop/package.json`，官方默认）：

| 顺序 | bundle                                             | 作用                                                         |
| ---- | -------------------------------------------------- | ------------------------------------------------------------ |
| 1    | `@deepseek-ai/dsh-base`                            | 共享核心（96 个 insert）                                     |
| 2    | `@deepseek-ai/dsh-web-app`                         | Web/desktop 表面 + 4 个 preset patch（90 insert、26 行覆盖） |
| 3    | `@deepseek-ai/dsh-experimental-agent-team-profile` | desktop 专属 Agent Teams（3 insert、4 disable）              |
| 4    | `@deepseek-ai/dsh-experimental-auto-review`        | desktop 专属自动审查（1 insert）                             |

desktop 没有独立的 bundle 包；desktop/web 差异靠 web-app 层里 `profileContext?.name === 'desktop'` 的条件行表达。

### 1.2 patch 行与「整段替换」

顶层是 YAML 数组，每项是一行 patch：

```yaml
- id: ui-chat # 必填；同一 profile 内同一 id 只能有一个非 insert 行
  name: '@deepseek-ai/...' # 可选
  config: { ... } # 整段替换该行的 config，不是深合并
- id: session-log-deepseek
  config:
    enabled: false
- id: something # 行级禁用
  disabled: true
- insert: # 插入新行/分组
    - id: ...
```

关键语义：

- **`config` 是整段替换**。写 `ui-theme.fontSize` 时如果不同时带上 `preference`，前者的其它键会被丢掉。GUI 必须「读当前有效 config → 改目标字段 → 整段写回」。
- **删除 `config` 键 = 恢复默认**（该行只剩 `id/name` 时整行删除），不要写一份 base 默认值的拷贝。
- 同一 id 重复出现、或该 id 被 home patch / `--patch` 覆盖时，DSH 设置页会**拒绝写入**（`config-editor/src/index.ts:139-141`）。DroidGear 应做同样的遮蔽检测。
- `!!js` 表达式（`sandbox-policy.mode`、`tools.mode`、`connection.trustedHosts` 等）必须原样保留，不能把求值结果写死。

### 1.3 可写字段：volatile

DSH 设置服务只把 Config 里标了 `.volatile()` 的字段投影成表单；非 volatile 字段在设置页不可见、写入被拒（`settings/settings/src/schema.ts:37-47`、`README.md:73-74`）。

对 DroidGear 的含义：

- 想与 DSH 设置页**共存不打架**：优先只写 volatile 字段。
- 写非 volatile 字段（如 `session-log-deepseek.maxBytes`、`compaction-basic.thresholdRatio`）不是非法，但 DSH 不会热应用，需要 reload/重启；DroidGear 应在 UI 上标注「重启后生效」。

### 1.4 secret

`role('secret')` 字段（如 `web-search-deepseek.apiKey`、`web-deepseek-api.apiKey`、`deepseek-account.requestHeaders`）在 remote 读取时被结构性摘除，值永不过 wire。DSH 官方页面把 API key 写进 `$DSH_HOME/.credentials.yaml` 的 `refs`，配置里只留 `apiKeyEnv` 引用名。

DroidGear 已有 credentials 页面（读写 `.credentials.yaml` refs，0600 + 原子写）。**不要**把 secret 字面量写进 `cordis.patch.yml`。

### 1.5 写入协议（DroidGear 需镜像）

DSH `config-editor` 的一次写入：

1. `withFileLock(join(profile.dir,'package.json'))`：`<profile>/package.json.lock` 由 `wx` 独占创建、记录 pid，默认等 2s，持有进程消失可接管（`packages/util/atomic-write/src/index.ts:216-273`）。
2. 重新读盘并 reconcile，确认 entry 仍存在、仍活跃。
3. 对完整 Config 做 schema 校验。
4. 用 `yaml` 的 `parseDocument` 编辑**现有文本**：只改目标行的 `config`，保留注释与 `!!js`。
5. 若下一份 config 与 inherited 深等 → 删除 `config` 键（恢复默认）。
6. 校验组合后该 entry 的最终 config 与期望深等；被更高层覆盖则拒绝。
7. `writeFileAtomic(path, text, { mode: 0o600 })`：同目录随机后缀临时文件 + rename。
8. `reconcileProfilePatches` 热更新；失败则回滚文件内容。
9. 全程包在 `hmr.runExclusive` 里。

DroidGear 现状（`src-tauri/crates/droidgear-core/src/storage.rs:3-19`）是固定 `path.with_extension("tmp")` + rename、无锁。单写者时够用，但与 DSH 设置页 / `dsh plugin` 并发时会丢更新。

## 2. DSH 自带设置 UI 的覆盖面

DSH 0.2.1 的「设置」不是一个页面，而是三处：

| 位置                       | 内容                                                                         |
| -------------------------- | ---------------------------------------------------------------------------- |
| 设置对话框（5 个 section） | 账号与余额、通用设置、模型、内置插件、Agent 预设                             |
| 插件主面板（侧栏「插件」） | 内置插件配置 4 张卡：终端、Agent 循环、子智能体、网页搜索 + bundle 安装/开关 |
| 设置 → 内置插件            | 只读插件清单                                                                 |

「通用设置」覆盖的字段（全部 volatile）：`locale.preference`、`ui-theme.{preference,fontSize,codeFontSize,terminalFontSize,textFontFamily,codeFontFamily,terminalFontFamily}`、`ui-chat.{transcriptView,collapseTiming,performanceUsage,linkOpening}`、`ui-conversation.busyEnter`、`ui-settings.enabled`、`permission.defaultPreset`、`session-log-deepseek.enabled`。

4 张插件卡：`bash-sandbox/pwsh-sandbox.{timeoutMs,maxOutputBytes}`、`agent-loop.maxParallelToolCalls`、`subagent.{maxDepth,maxActiveSubagents}` + `subagent-model-selection-settings.{enabled,allowedModels}`、`web.{searchProvider,fetchProvider}` + `web-search-deepseek.*` + `web-deepseek-api.*`。

模型页覆盖 `llm-pi-ai.providers.<id>`、`llm-deepseek.*`、`llm-deepseek-account.*`（key 走 credentials）。

**结论**：DSH 已有 UI 的字段可以直接 1:1 抄控件；DroidGear 的增量价值在于（a）把这些页面搬进桌面端一站式管理，（b）覆盖 DSH 没有 UI 的 B/C 档字段。

## 3. 可视化清单（分档）

### 3.1 A 档：第一批直接做

| entry id                                           | 字段（类型/默认）                                                                                                                                                                                        | 建议控件                                       | 备注                                                                                          |
| -------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `agent-default-model`                              | `provider`（string，当前 `deepseek-official`）、`model`（string，当前 `deepseek-flash`）、`reasoningEffort`（当前 `high`）                                                                               | 联动 Select（provider → model）+ effort Select | 新会话默认模型；volatile。provider/model 建议从已配置 providers + 模型列表填充                |
| `llm-pi-ai`                                        | `providers.<id>.{displayName,apiKeyEnv,api,baseURL,models[]}`                                                                                                                                            | 已有 ProviderDialog，补 0.2.1 新字段即可       | 自定义模型路由唯一入口                                                                        |
| `llm-deepseek` / `llm-deepseek-account`            | `baseURL`、`thinking`（enabled/disabled）、`reasoningEffort`（off/low/high/max，默认 high）、`maxTokens`（默认 256000）                                                                                  | URL / Select / Number                          | 官方模型端点与思考策略                                                                        |
| `web` + `web-search-deepseek` + `web-deepseek-api` | `searchProvider`（当前 `deepseek-official`）、`fetchProvider`（当前 `http`）；`model`（默认 `deepseek-v4-flash`）、`maxUses`（默认 5）、`count`（1..50，默认 10）、`baseURL`、`apiKeyEnv`                | 开关 + Select + Number                         | key 走 credentials，配置只写 `apiKeyEnv`                                                      |
| `sandbox-policy` + `permission` + `approval`       | `mode`（read-only/workspace-write/danger-full-access）、`defaultPreset`、`policy`（ask/never）                                                                                                           | 三选一分段 + 二次确认                          | 安全边界；`mode` 当前是 `!!js process.env.DSH_PERMISSION_MODE ?? 'workspace-write'`，不要写死 |
| `locale`                                           | `preference`（zh/en；当前 zh）                                                                                                                                                                           | Select                                         | volatile，即时生效                                                                            |
| `ui-theme`                                         | `preference`（light/dark/system，默认 system）、`fontSize`（14，10-22）、`codeFontSize`（11，10-16）、`terminalFontSize`（13，10-20）、三个字体族                                                        | 主题卡 + 步进 + 文本                           | 全部 volatile                                                                                 |
| `ui-chat`                                          | `transcriptView`（compact/standard/detailed/verbose，默认 detailed，当前 standard）、`collapseTiming`（completion/next-input）、`performanceUsage`（compact/detailed）、`linkOpening`（sidebar/new-tab） | 4 个 Select                                    |                                                                                               |
| `ui-conversation`                                  | `busyEnter`（queue/steer，默认 queue）                                                                                                                                                                   | Select                                         |                                                                                               |
| `ui-settings`                                      | `enabled`（开发者工具，默认 true）                                                                                                                                                                       | Switch                                         | 当前 true                                                                                     |
| `session-log-deepseek`                             | `enabled`（默认 true，可关闭）                                                                                                                                                                           | Switch                                         | 隐私开关；注意该行**不是** disabled，是 config 里 false                                       |
| `bash-sandbox`                                     | `timeoutMs`（schema 默认 120000，base/config 60000）、`maxOutputBytes`（默认 64000）                                                                                                                     | Number（秒/字节人性化）                        | Windows 改用 `pwsh-sandbox`                                                                   |
| `agent-loop`                                       | `maxParallelToolCalls`（默认 10）                                                                                                                                                                        | Number                                         | volatile                                                                                      |
| `subagent`                                         | `maxDepth`（默认 1，0=禁用）、`maxActiveSubagents`（默认 8）                                                                                                                                             | Number                                         | volatile                                                                                      |
| `subagent-model-selection-settings`                | `enabled`（默认 false）、`allowedModels[]`                                                                                                                                                               | Switch + 多选                                  | 开启后必须至少一个路由                                                                        |
| `agent-preset-registry`                            | `default`（当前 standard）、`selectedDefault`                                                                                                                                                            | Select                                         | 决定新会话工具集                                                                              |
| `compaction-basic`（preset 层）                    | `auto`（true）、`thresholdRatio`（0.8）、`headroomTokens`（65536）、`retainRatio`（0.16）                                                                                                                | Switch + Number                                | 在 `preset-*.config.plugins[]` 里，需整体替换 preset 行                                       |

### 3.2 B 档：折叠进「高级」

- 模型细节：`llm-deepseek.{streamIdleTimeoutMs,retryPolicy,models[],defaultContextWindow}`、`llm-pi-ai.providers.*.{api,input,reasoningEfforts,timeoutMs,retryPolicy}`
- 搜索/抓取：`web-fetch-http.{timeoutMs,maxRedirects,userAgent}`、`web-deepseek-api.sendOriginalQuestion`
- 会话/上下文：`session-title-llm.{targetWords,targetCjkCharacters}`、`spill-policy.maxInlineTokens`、`session-query-sqlite.openAt`（默认 never，全文检索默认关闭）、`session-reference.*`、`file-reference-local.*`、`attachment-local.{maxImageBytes,maxImagesPerMessage}`
- 权限/沙箱细节：`permission.presets`（预设表）、`bash-sandbox.{maxTimeoutMs,maxSpillBytes,graceMs}`
- 子代理/团队：`agent-team.{maxMembers,maxTasks}`（desktop 专属）
- 工具/preset 层：`tool-web.{fetch,searchTimeoutMs}`、`tool-ask-user.timeout`、`tool-todo.allowParallelInProgress`、`tool-fs.{read,write,edit}`、`agent-instructions.maxBytes`、`persona.prefix`（均在 `preset-*` 行内）
- 遥测/隐私：`session-telemetry-otel.mode`（默认 FEEDBACK_ONLY）、`product-analytics.enabled`（desktop）、`message-feedback.maxNoteBytes`
- 插件市场：`plugin-market.indexes`、`ui-plugin-manager.{registryProbeEnabled,registryProbeTimeoutMs,registryProbeCacheTtlMs}`

### 3.3 C 档：只读或不暴露

路径、端口、鉴权、持久化后端、平台内部：`webserver.{host,port,tls,compression*}`、`deepseek-account.*`、`credentials.{path,dshHome,debounceMs}`、`storage-json.root`、`storage-domain.*`、`session-persistence-jsonl.*`、`session-projection-cache.*`、`sandbox.runnerCommand`、`working-directory.defaultDirectory`、`terminal-controller.*`、`workspace-files.*`、`desktop-product-telemetry.*`、`ui-settings-account.*`（引导进度，内部）。

展示应只读（或完全不展示），写错会导致启动失败/数据错位。

## 4. 只读诊断面

这些不是「配置」，但适合在 Dsh 页展示：

| 数据                        | 来源                                                       | 说明                                                                              |
| --------------------------- | ---------------------------------------------------------- | --------------------------------------------------------------------------------- |
| profile / bundle / 活动条目 | 组合 `package.json` + 各 bundle patch + profile/home patch | 有效清单需要自己组合（`!!js` 只能原样展示）；运行态 fiber 信息只有 DSH 进程内准确 |
| 配置路径与来源层            | `get_dsh_config_status` 扩展                               | 显示 profile patch / home patch / `settings.yaml.imported` 是否存在               |
| credentials refs            | `.credentials.yaml`                                        | 只显示键名 + 是否已配置，不读值                                                   |
| sessions                    | `~/.dsh/sessions` + `storages/session_projcache`           | DroidGear 已实现                                                                  |
| Agent presets               | preset-\* 行（不是 `~/.dsh/.agent-presets`）               | 0.2.1 不读 `.agent-presets` 目录，只能当遗留目录提示                              |
| 版本/运行状态               | `desktop-runtime.json`、执行中的进程                       | 0.1 vs 0.2 配置差异提示                                                           |

## 5. DroidGear 现状与缺口

已有：`DshConfigPage`（profile 维度 desktop/web + 模型服务商 CRUD + credentials + 配置状态）、`DshSessionsPage`、TUI 对等页面。

与 0.2.1 的差距（按优先级）：

| #   | 缺口                                                  | 证据                                                                                                             | 影响                                                                            |
| --- | ----------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| 1   | 写入无 DSH 锁、固定 `.tmp` 文件名                     | `storage.rs:15-16`；DSH 用 `<profile>/package.json.lock` + 随机 tmp（`atomic-write/src/index.ts:80-95,238-273`） | 与 DSH 设置页/`dsh plugin` 并发时丢更新                                         |
| 2   | 不检测 home patch / `--patch` 遮蔽                    | DSH 会拒绝写（`config-editor/src/index.ts:139-141`）                                                             | 「保存成功但无效」                                                              |
| 3   | 仍会回退读写 `settings.yaml`（无 profile 时）         | `dsh.rs:791-819`                                                                                                 | 0.2.1 下无效，且下次启动可能被一次性导入污染 profile patch                      |
| 4   | profile 判据是「有 package.json 或 cordis.patch.yml」 | `dsh.rs:233-253`；DSH 必读 `package.json`（`profile.ts:740-806`）                                                | 只有 patch 的目录会被列出但 DSH 无法加载                                        |
| 5   | 只处理 `llm-pi-ai` 一个条目                           | `dsh.rs:33-36`                                                                                                   | 无法覆盖其它 80+ 个可配条目                                                     |
| 6   | 空/注释-only patch 文件会被 DSH 拒绝启动              | `app-boot/src/index.ts:520-548`                                                                                  | DroidGear 清空保存时必须写 `[]`                                                 |
| 7   | `~/.dsh/profiles/node_modules` 是 0.1.0-rc.6 旧树     | 实测 343MB，desktop 当前不从它解析，但它是 Node 回退层                                                           | 若 patch 插入 0.2.1 已删除的包名（旧树独有 17 个）会加载 0.1 代码；建议提示归档 |

0.1 → 0.2 的配置格式变化（DroidGear 展示旧配置时需要提示迁移）：

- `tools.mode: both` 在 0.2.1 被拒绝，只接受 `native` / `ptc`。
- 子代理 provider 的 `backgroundMode` 已移除。
- agent plane 移入 preset：宿主层的 `tool-bash`/`tool-fs`/`compaction-basic` 等在 Web/desktop 被 `disabled`，写它们的 config 不生效，必须改 `preset-*.config.plugins[]`。
- `schedule-bundle` 退役；模型字段 `imageDetail` → `imagePixelBudget`。
- Azure 路由键 `azure-openai-responses` → `azure`（`api` 标识符不变）。

## 6. 实现方案

### 6.1 后端

新增一组通用 patch 命令（Rust core），与现有 `dsh` 模块并列：

- `read_dsh_profile_patch(profile) -> { entries: [{id, name, config, disabled, sourceLayer}], homePatch, overlays }`：组合 bundle + profile + home 层，标注每个字段来自哪层；`!!js` 原样保留（不反序列化求值）。
- `save_dsh_patch_entry(profile, id, config, expectedHash)`：读-合并-写；`config: null` 表示删除该行 config。
- `set_dsh_patch_disabled(profile, id, disabled)`。
- 写入实现复用/升级 `storage::atomic_write`：随机后缀临时文件 + `flock`/`<profile>/package.json.lock`（与 DSH 同名）+ 0600。
- schema 快照：由 DSH 源码生成一份 `dsh-config-schema.json`（字段名/类型/默认/枚举/volatile/secret/说明/来源），放在 `droidgear-core` 里 `include_str!`。生成脚本可以照搬 DSH 的 `scripts/gen-config-catalog.ts` 思路，在 DSH 升级时重跑。

### 6.2 前端

Dsh 页保持现有 profile 维度（desktop/web），每个 profile 下分栏：

1. **概览**：配置状态（profile patch / home patch / 旧 settings.yaml / credentials）、版本、风险提示（home patch 遮蔽、旧 node_modules）。
2. **模型**：现有 ProviderCard/Dialog 扩展。
3. **通用设置**：语言、主题、聊天展示、开发者工具。
4. **Agent**：默认模型、agent preset、agent loop、subagent、权限模式。
5. **网络**：web search、fetch、代理提示（代理不是配置字段，指向 `$DSH_HOME/.env`）。
6. **高级**：schema 驱动的字段表（按 entry 分组，标注 volatile/需重启），只开放白名单字段。
7. **只读诊断**：插件清单、presets、credentials 键名、sessions 入口。

通用表单不要做成「全自动 renderer」。DSH 自己都没有 auto renderer；按 A 档手写页面 + B 档按 entry 分组的白名单字段表更稳。

### 6.3 写入规则

- 只写 `profiles/<profile>/cordis.patch.yml`；home patch 只读。
- 写前重读 + 比对内容哈希；不一致时提示「配置已被 DSH 或其它工具修改」并要求刷新（DroidGear 没有 DSH 的进程内 revision）。
- 保留注释与 `!!js`：用 YAML Document 级编辑，不要 stringify 全文件。现有 `dsh.rs` 的 `split_patch_document` 思路可以复用/扩展。
- 删除字段 = 从 `config` 里删键并让下层值生效；恢复整条默认 = 删除该行 config。
- 保存后提示「DSH 正在运行时会自动热加载；非 volatile 字段需重启」。
- 空文件写 `[]`，不要留空/纯注释。

### 6.4 TUI 对等

按 AGENTS.md 规则 9，GUI 每加一个配置区，TUI 同步加对应 `keys_dsh.rs` 入口与表单（复用同一组 Rust core 命令）。TUI 不需要完整复刻所有控件，但至少要能读写同一批字段。

## 7. 分阶段

| 阶段 | 内容                                                                                         |
| ---- | -------------------------------------------------------------------------------------------- |
| P1   | 通用 patch 读写命令 + 锁/原子写升级；通用设置页（语言/主题/聊天/开发者工具）；默认模型选择器 |
| P2   | 插件卡片：Agent 循环、子智能体、终端、网页搜索；权限三件套；会话日志开关                     |
| P3   | 模型页对齐 0.2.1（pi-ai 新字段、DeepSeek 目录、credentials 引用）；高级字段表                |
| P4   | 插件管理（bundle 开关/安装，走 `dsh plugin`）、preset 编辑器、只读诊断面板                   |

## 8. 风险与开放问题

- **并发写**：DSH 运行中用户可能在设置了改同一字段；锁 + 哈希检测是硬要求。
- **home patch 遮蔽**：`~/.dsh/cordis.patch.yml` 一旦覆盖同 id，profile 层写入无效；保存前必须检查。
- **secret**：不要在 UI/日志/遥测里出现 key；只写 `apiKeyEnv`，key 只进 `.credentials.yaml`。
- **`!!js`**：任何「读出来求值再写回」的实现都会破坏环境变量/平台条件，必须拒绝这类字段的写操作。
- **schema 漂移**：DSH rc 之间会删改字段；schema 快照要随 DSH 版本更新，并在页面显示基线版本。
- **桌面实例不可 API 复用**：不要尝试读 Electron Cookies/Keychain 伪造鉴权。
- **旧 node_modules**：检测到 `~/.dsh/profiles/node_modules` 版本与安装版不一致时提示归档，避免回退加载 0.1 代码。

## 9. 实现

### 9.1 后端

- `src-tauri/crates/droidgear-core/src/dsh_config.rs`：通用 profile patch 读写。
  - `read_dsh_config_snapshot(profile)`：组合 bundle（来自 schema 快照）+ profile patch + home patch，返回每个 entry 的层来源、生效 config、字段值/默认值、`overridden`/`shadowedByHome`、`hasJs`、migration warnings。
  - `save_dsh_config_entry(profile, entry_id, config, expected_hash)`：整段替换 config；`config = null` 删除覆盖（恢复默认）。
  - `set_dsh_config_entry_disabled(profile, entry_id, disabled, expected_hash)`。
  - 写入协议：`<profile>/package.json.lock`（与 DSH 同名、2s 等待、30s 陈旧接管）→ YAML Document 级编辑（注释保留）→ 随机后缀临时文件 + 0600 + rename；`!!js` 表达式按 `$js` 标记往返；其他自定义 tag 拒绝重写；home patch 遮蔽拒绝；`expected_hash` 冲突返回 `DSH_CONFIG_CONFLICT`。
  - secret：snapshot 里 top-level secret 字段替换为 `{"$secret": true}`；保存时该标记表示「保持继承值」，明文 secret 拒绝写入。
- `src-tauri/src/commands/dsh_config.rs` + `src/lib/bindings.ts`：三个 Tauri 命令。
- `DshProviderConfig.reasoning`（pi-ai 路由默认思考强度，`off/minimal/low/medium/high/xhigh/max`，空 = 跟随 provider 默认；后端 normalize + 校验）。这是 DSH 生成 `defaultEffort` 的唯一来源：不写它，模型选择器就显示「Default」。
- `DshModel.input`（`text` / `image`）：保存时按 model id/alias 从 DroidGear registry 自动填充（如 `deepseek-flash`→text+image、`deepseek-v4-pro`→text），非法值拒绝。自定义 pi-ai 路由没有 installed catalog 可继承，DSH 会把未声明的模型当成 route `defaultInput` = 仅 `text`，所以不写它图像请求会被拒（`UNSUPPORTED_CONTENT`）。
- `src-tauri/crates/droidgear-core/src/storage.rs`：`atomic_write` 改为唯一临时文件名；新增 `atomic_write_private`（0600）。

### 9.2 Schema 快照

`src-tauri/crates/droidgear-core/res/dsh-config-schema.json` 由脚本生成：

```bash
node scripts/gen-dsh-config-schema.mjs --dsh <deepseek-harness>            # 重新生成
node scripts/gen-dsh-config-schema.mjs --dsh <deepseek-harness> --check   # 校验已提交文件与源一致（DSH 升级后跑）
```

快照包含：四个 bundle 的 entry/config/disabled（按 profile × OS 求值）、每个 config entry 的顶层字段（kind/default/enum/volatile/secret/tier/description）。升级 DSH 后必须重跑并提交新快照，否则 UI 字段与 schema 校验会落后于运行时。

### 9.3 前端

- `DshConfigPage`：在现有 desktop/web profile 维度内增加 7 个 tab：概览、模型、通用、Agent、网络、高级、诊断。
- `src/components/dsh/config/`：通用表单组件（SettingsSection/SettingsRow/EnumField/NumberField/SwitchField/TextField/徽标/PageHeader）。
- `EntrySettingsCard`：按 entry 渲染字段并 staged 保存；`$js`/secret/json/C 档只读，非 volatile 显示「重启后生效」。
- Models tab 的 ProviderDialog：新增「默认思考强度」Select（新建/未显式设置时，若该路由所有模型已知支持 `high` 则自动建议 `high`，与官方 DeepSeek 行为对齐；部分模型不支持时显示警告），以及模型行的「设为默认模型」星标——保存时把 `{provider, model, reasoningEffort}` 写入 `agent-default-model` 条目（复用通用 `saveEntry`，带 patch hash 乐观锁）。
- 模型行新增「输入类型」：文本固定、图像可勾选；未手动改过时随 model id/alias 按 registry 自动建议（`deepseek-flash` 自动含 image），手动切换后不再被覆盖，自定义网关不支持图像时可取消。
- 官方 DeepSeek 卡（Models tab 顶部）：给不使用 DSH 账号登录的用户配置 `deepseek-official` 路由的 API Key。读取 `llm-deepseek` entry 的 `apiKeyEnv`（默认 `DEEPSEEK_API_KEY`），key 写入 `~/.dsh/.credentials.yaml` 的 refs；支持自定义引用名（同时回写 `llm-deepseek.apiKeyEnv`）与清除（回退环境变量）。TUI 在 Dsh 列表按 `o` 打开同一配置。
- API Key 区域：「凭据引用名」（`providers.<id>.apiKeyEnv`）默认自动推导为 `<PROVIDER_ID>_API_KEY`，主表单只保留 key 值；仅在需要复用已有环境变量/自定义引用名时展开「自定义」。key 值写入 `~/.dsh/.credentials.yaml` 的 `refs[引用名]`。DSH 的凭据解析优先级是 **credentials 存储值 > 启动环境变量 > 项目 `.env` > `$DSH_HOME/.env`**（存储值优先，环境变量只是回退）；清空 key 保存会删除该 ref 并回退到环境变量。TUI 在 `apiKeyEnv` 留空时同样自动推导。
- `src/store/dsh-store.ts`：snapshot + `loadSnapshot`/`saveEntry`/`toggleEntryDisabled`，冲突/遮蔽错误转成提示。

### 9.4 TUI

`src-tauri/crates/droidgear-tui/` 增加 Dsh 配置视图（`keys_dsh_config.rs`）：显示 profile/版本/warnings，entries 列表，`d` 切换 disabled、`r` 刷新、Esc 返回。

### 9.5 测试

```bash
cd src-tauri && cargo test -p droidgear-core dsh_config
npx vitest run src/components/dsh
cd src-tauri && cargo test -p droidgear-tui
```

## 10. 数据来源与再生成

Schema 快照与文档结论来自 DSH 0.2.1-rc.2 源码（`docs/config-catalog.zh.md` 的生成式 TS 声明、四个 bundle 的 patch 文件、各包的 schemastery 定义与 JSDoc）；本机 profile 实测只用于验证行为，不参与生成。

升级 DSH 后：

1. 更新生成器读取的源码 checkout（`--dsh` 或 `DSH_SOURCE_DIR`）；
2. 重新生成并提交 `res/dsh-config-schema.json`；
3. 跑 `node scripts/gen-dsh-config-schema.mjs --dsh <checkout> --check` 与 `cargo test -p droidgear-core dsh_config`；
4. 若生成器报告新的/删除的 bundle 或字段，同步检查第 3 节的档位表与 UI 白名单。
