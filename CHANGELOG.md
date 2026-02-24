# Changelog

## v0.4.0

**New Features / 新功能**

- Add Ollama channel support with auto-detection / 添加 Ollama 频道支持，支持自动检测
- Add OpenAI/Gemini provider templates and fix channel import protocol inference / 添加 OpenAI/Gemini 服务商模板，修复频道导入协议推断

## v0.3.9

**New Features / 新功能**

- Add General channel type with API key auth / 添加通用频道类型，支持 API 密钥认证

## v0.3.8

**New Features / 新功能**

- Add OpenAI Responses message type support for OpenClaw / OpenClaw 添加 OpenAI Responses 消息类型支持

**Bug Fixes / 问题修复**

- Use bash instead of sh for OpenCode install command / OpenCode 安装命令使用 bash 替代 sh
- Fix ugly close button / 修复关闭按钮样式问题
- Reduce white splash screen / 减少白色闪屏
- Reduce UI jump glitch / 减少 UI 跳动问题
- Fix CSS build warning / 修复 CSS 构建警告

## v0.3.7

**New Features / 新功能**

- Add OpenClaw path configuration to system settings / 在系统设置中添加 OpenClaw 路径配置

**Bug Fixes / 问题修复**

- Auto-refresh UI after path save/reset / 路径保存/重置后自动刷新 UI

## v0.3.6

**Refactoring / 重构**

- Complete rewrite of Codex provider management with structured architecture / 完全重写 Codex 服务商管理，采用结构化架构
- Unify Codex Providers layout with OpenCode/OpenClaw / 统一 Codex 服务商布局与 OpenCode/OpenClaw 一致

**Documentation / 文档**

- Refactor AGENTS.md with progressive disclosure and split into docs/agents/ / 重构 AGENTS.md，采用渐进式披露并拆分到 docs/agents/

## v0.3.5

**New Features / 新功能**

- Auto-save providers like OpenClaw, remove Reset/Save buttons in OpenCode / OpenCode 中自动保存服务商（类似 OpenClaw），移除重置/保存按钮
- Add models support with multi-select channel import in OpenCode / OpenCode 中添加模型支持，支持多选频道导入
- Add Import from Channel with model protocol inference in OpenCode / OpenCode 中添加从频道导入功能，支持模型协议推断

**Bug Fixes / 问题修复**

- Correct baseURL field name and add /v1 for anthropic protocol in OpenCode / 修复 OpenCode 中 baseURL 字段名称并为 anthropic 协议添加 /v1

## v0.3.4

**New Features / 新功能**

- Validate default model before applying OpenClaw profile / 应用 OpenClaw 配置前验证默认模型
- Promote Import from Channel to provider-level operation in OpenClaw / OpenClaw 中将从频道导入提升为服务商级别操作
- Require at least one model when saving OpenClaw provider / 保存 OpenClaw 服务商时要求至少一个模型
- Add Exa and Context7 MCP presets with stdio/http variants / 添加 Exa 和 Context7 MCP 预设，支持 stdio/http 变体
- Auto-sync model display name when entering model ID in OpenClaw / OpenClaw 中输入模型 ID 时自动同步显示名称

**Bug Fixes / 问题修复**

- Isolate per-channel fetch state to fix infinite retry loop / 隔离每个频道的获取状态以修复无限重试循环

## v0.3.3

**New Features / 新功能**

- Add CLI Proxy API channel type support / 添加 CLI Proxy API 频道类型支持
- Add ChannelModelPicker for quick model import from channels / 添加 ChannelModelPicker 用于从频道快速导入模型

**Bug Fixes / 问题修复**

- Relax displayName validation to allow hyphen/underscore separators / 放宽 displayName 验证以允许连字符/下划线分隔符
- Fix specs page delete race condition and add error recovery / 修复 specs 页面删除竞态条件并添加错误恢复
- Fix OpenClaw profile apply policy / 修复 OpenClaw 配置应用策略

## v0.3.2

**New Features / 新功能**

- OpenClaw streaming settings support / OpenClaw 流式设置支持
- OpenClaw models providers more options / OpenClaw 模型服务商更多选项

**Bug Fixes / 问题修复**

- Fix OpenClaw apply mode / 修复 OpenClaw 应用模式

## v0.3.1

**New Features / 新功能**

- OpenClaw provider/model configuration support / OpenClaw 服务商/模型配置支持
- OpenClaw config improvements / OpenClaw 配置改进

## v0.3.0

**New Features / 新功能**

- Auto detect channel type / 自动检测频道类型
- Detect display name use droid official name as prefix / 检测显示名称时使用 Droid 官方名称作为前缀

**Bug Fixes / 问题修复**

- Add base url to dedup / 添加 baseUrl 到去重逻辑
- Fix isWindows detection / 修复 isWindows 检测

## v0.2.9

**Bug Fixes / 问题修复**

- Fix all platform tips for skipping login of droid / 修复所有平台跳过 Droid 登录的提示
- Fix i18n CN message of models.alreadyAddedForKey / 修复 models.alreadyAddedForKey 的中文翻译
- Fix fetch model action on copy/edit mode / 修复复制/编辑模式下获取模型的操作

## v0.2.8

**New Features / 新功能**

- Support custom config path for WSL / 支持 WSL 自定义配置路径 #10

## v0.2.7

**New Features / 新功能**

- Codex CLI config support / Codex CLI 配置支持
- [sub2api] Display remote group name for API keys / [sub2api] 显示远程分组名称

## v0.2.6

**New Features / 新功能**

- Universal multi models component for byok and channels / 通用多模型组件，支持 BYOK 和频道
- Add new preset mcp server exa to replace droid websearch / 添加新的预设 MCP 服务器 exa 替代 droid websearch

**Bug Fixes / 问题修复**

- Auto flush saveModels action #8 / 自动刷新 saveModels 操作 #8

## v0.2.5

**New Features / 新功能**

- Cmd/Ctrl + Shift + [ to switch to previous tab / Cmd/Ctrl + Shift + [ 切换到上一个标签页

**Bug Fixes / 问题修复**

- Fix session list not refreshing after deletion / 修复删除会话后列表不刷新的问题 #7

## v0.2.4

**New Features / 新功能**

- Support Cmd/Ctrl+1/2/3..0 to switch terminal / 支持 Cmd/Ctrl+1/2/3..0 切换终端
- Delete session / 删除会话
- Update check improvements and downloading progress / 更新检查改进和下载进度显示
- Use custom ActionButton and ActionDropdownMenuItem / 使用自定义 ActionButton 和 ActionDropdownMenuItem
- Allow Windows user specify custom terminal command / 允许 Windows 用户指定自定义终端命令

**Bug Fixes / 问题修复**

- Ensure locale is set for proper CJK character display / 确保设置区域以正确显示 CJK 字符
- Wrong selection active if IME active / 修复输入法激活时选择错误
- IME compatibility / 输入法兼容性修复

## v0.2.3

**New Features / 新功能**

- Ctrl/Cmd + W to close terminal tab / Ctrl/Cmd + W 关闭终端标签页
- Show toggle left sidebar button / 显示切换左侧边栏按钮
- Add Snippets on Terminal pages / 终端页面添加代码片段功能
- Default use directory name as Terminal name / 默认使用目录名作为终端名称

**Bug Fixes / 问题修复**

- Ctrl/Cmd+W only bind in Terminal page / Ctrl/Cmd+W 仅在终端页面绑定
- Allow empty derived terminal / 允许空的派生终端
- Big performance improvement for terminal loading / 终端加载性能大幅提升
- Update status checking / 更新状态检查
- Auto generate display name / 自动生成显示名称
- Rename terminal on Windows / 修复 Windows 下终端重命名
- Use windows custom config for github actions / GitHub Actions 使用 Windows 自定义配置

## v0.2.2

**Bug Fixes / 问题修复**

- Fix TERM/COLORTERM environment variable injection / 修复 TERM/COLORTERM 环境变量注入问题

## v0.2.1

**New Features / 新功能**

- Terminal support open derived sub window / 终端支持打开派生子窗口
- Support shift+enter on macOS and ctrl+shift+c/v on Windows/Linux / macOS 支持 shift+enter，Windows/Linux 支持 ctrl+shift+c/v

**Bug Fixes / 问题修复**

- Auto focus and selection while rename Terminal name / 重命名终端时自动聚焦和选中
- Allow use dot in model name / 允许模型名称中使用点号
- Terminal bottom style and model maxTokens step size / 修复终端底部样式和模型 maxTokens 步长
- Save and restore window state / 保存和恢复窗口状态
- Remove custom env to inherit system environment / 移除自定义环境变量以继承系统环境

## v0.2.0

**New Features / 新功能**

- Terminal support OSC 9 notifications / 终端支持 OSC 9 通知
- Use single button to toggle session list/grouped view / 使用单个按钮切换会话列表/分组视图
- Sessions hide empty groups / 会话隐藏空分组
- Use different indicators for Terminal active and notification / 终端活动和通知使用不同的指示器
- Terminal add copy-on-select / 终端添加选中即复制功能
- Fix left panel width / 修复左侧面板宽度

## v0.1.9

**New Features / 新功能**

- Terminal add reload and remove state control / 终端添加重新加载和移除状态控制
- Custom terminal font / 自定义终端字体
- Sessions support hiding empty / 会话支持隐藏空会话
- Terminal inject envs and add force dark mode / 终端注入环境变量并添加强制深色模式

## v0.1.8

**New Features / 新功能**

- Save and restore terminal status / 保存和恢复终端状态
- Embedded terminals support / 支持嵌入式终端
- More powerful session follow mode and toggle thinking expansion / 更强大的会话跟随模式，支持切换思考展开状态

## v0.1.7

**New Features / 新功能**

- Support droid sessions / 支持 Droid 会话

## v0.1.6

**New Features / 新功能**

- Copy spec full path / 复制规格完整路径
- Auto set displayName / 自动设置显示名称

## v0.1.5

**New Features / 新功能**

- Add tips about websearch tool-call issue if skip login / 添加跳过登录时 websearch 工具调用问题的提示
- Add environment variable conflict hinter / 添加环境变量冲突提示
- Use toast instead of confirm while check update / 检查更新时使用 toast 代替确认框

**Bug Fixes / 问题修复**

- Prevent the use of system model names / 禁止使用系统模型名称
- Add more special chars for droid display name / 为 Droid 显示名称添加更多特殊字符支持
- Fix scroll top if switch spec / 修复切换规格时滚动到顶部
- Fix wrong command in windows / 修复 Windows 下的错误命令
- Disable autoCorrect autoComplete autoCapitalize spellCheck / 禁用自动更正、自动完成、自动大写和拼写检查

## v0.1.4

**New Features / 新功能**

- Add OpenCode support for AI development / 添加 OpenCode 支持，用于 AI 开发
- Load and save OpenCode providers/auth to profiles / 加载和保存 OpenCode 服务商/认证到配置文件
- Click Spec title to rename / 点击规格标题可重命名
- Use resizable dialog for OpenCode provider / OpenCode 服务商使用可调整大小的对话框

**Bug Fixes / 问题修复**

- Fix window size and tab width issues / 修复窗口大小和标签宽度问题
- Fix spec render causing window overflow / 修复规格渲染导致窗口溢出
- Fix try skip login behavior changed / 修复尝试跳过登录行为变更

## v0.1.3

**New Features / 新功能**

- Support spec selection and edit mode / 支持规格选择和编辑模式
- Add hourly check update / 添加每小时检查更新
- Updater add release notes display / 更新器添加发布说明显示
- Support new-api OpenAI models detection / 支持 new-api OpenAI 模型检测

**Bug Fixes / 问题修复**

- Do not empty API URL if switch provider type / 切换服务商类型时不清空 API URL

## v0.1.2

**New Features / 新功能**

- Add MCP presets / 添加 MCP 预设
- MCP server management / MCP 服务器管理
- Add more Droid settings options / 添加更多 Droid 设置选项
- Disallow parentheses in alias/display name / 禁止在别名/显示名称中使用括号
- Auto-generate ID and index consistent with Factory Droid rules / 自动生成 ID 和索引，与 Factory Droid 规则一致

**Bug Fixes / 问题修复**

- Fix sub2api OpenAI models no longer append /v1 / 修复 sub2api OpenAI 模型不再追加 /v1
- Fix default fill maxOutputTokens / 修复默认填充 maxOutputTokens

## v0.1.1

**New Features / 新功能**

- Cloud session sync toggle / 云端会话同步开关
- Add installation instructions section / 添加安装说明区域
- Ensure display name uniqueness / 确保显示名称唯一性

**Bug Fixes / 问题修复**

- Fix default tab to droid / 修复默认标签页为 droid

## v0.1.0

**New Features / 新功能**

- Add about page, remove examples / 添加关于页面，移除示例
- Support sub2api antigravity platform / 支持 sub2api 的 antigravity 平台

**Bug Fixes / 问题修复**

- Fix style lint issues / 修复样式 lint 问题
- Fix text center alignment / 修复文本居中对齐
- Fix fetch all keys pagination issue / 修复获取所有密钥分页问题

## v0.0.9

**New Features / 新功能**

- Add tip for using /model to switch models / 添加使用 /model 切换模型的提示
- Support setting default model and mark current default / 支持设置默认模型并标记当前默认
- Enhance dialog drag functionality / 增强对话框拖拽功能

**Bug Fixes / 问题修复**

- Fix fetch keys issue / 修复获取密钥的问题
- Fix channel save path error (does not affect Droid settings) / 修复频道保存路径错误（不影响 Droid 设置）
- Fix losing changes when switching Droid tabs / 修复切换 Droid 标签页时丢失更改的问题
- Fix dark mode color issue when rendering code blocks / 修复渲染代码块时的深色模式颜色问题

## v0.0.8

**New Features / 新功能**

- Specs panel supports rename, delete, save as operations / Specs 面板支持重命名、删除、另存为操作

**Bug Fixes / 问题修复**

- Disable global context menu / 禁用全局右键菜单

## v0.0.7

**New Features / 新功能**

- Add Specs panel to view spec files in ~/.factory/specs directory / 新增 Specs 面板功能，支持查看 ~/.factory/specs 目录下的规格文件
- Support Markdown format rendering for spec file content / 支持 Markdown 格式渲染规格文件内容

## v0.0.6

**New Features / 新功能**

- Add skip login helper feature / 新增跳过登录辅助功能

**Bug Fixes / 问题修复**

- Fix binary upload issue / 修复二进制上传问题
- Fix Anthropic model fetch: third-party proxy services now support OpenAI-style Bearer token authentication / 修复 Anthropic 模型获取：第三方代理服务现支持 OpenAI 风格的 Bearer token 认证

## v0.0.5

**New Features / 新功能**

- Copy model feature / 复制模型功能
- Filter and batch delete feature / 筛选和批量删除功能
- Import/export configuration feature / 导入导出配置功能
- Add sub2api platform support / 新增 sub2api 平台支持
- Support selecting provider when adding models from channel / 从频道添加模型时支持选择服务商

**Bug Fixes / 问题修复**

- Fix codex/gemini platform support (gemini now only supports v1beta/models) / 修复 codex/gemini 平台支持（gemini 现仅支持 v1beta/models）
- Fix API path issue when no platform / 修复无平台时 API 路径问题
- Fix model scroll issue on import / 修复导入时模型滚动问题

## v0.0.4

**New Features / 新功能**

- Version check and upgrade prompt / 版本检查和升级提示
- Disable quick panel by default / 默认禁用快捷面板

**Bug Fixes / 问题修复**

- Fix model key uniqueness issue / 修复模型 key 唯一性问题
- Fix interface overflow layout issue / 修复界面溢出布局问题
- Fix unsaved confirmation prompt on switch / 修复切换时未保存确认提示
- Fix channel name mapping issue / 修复频道名称映射问题
