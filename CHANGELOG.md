# Changelog

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
