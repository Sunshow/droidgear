# DroidGear

[English](README_EN.md)

用于在 [Factory Droid](https://factory.ai) 中配置自定义 AI 模型的桌面应用，支持 BYOK（自带密钥）。

## 安装说明

### macOS

由于应用未经 Apple 签名，下载后可能被 Gatekeeper 阻止运行。请执行以下命令解除限制：

```bash
xattr -cr /Applications/DroidGear.app
```

### Windows / Linux

直接运行安装程序即可。

## 功能特性

- **多服务商支持** - 配置来自 Anthropic、OpenAI 或任何通用 Chat Completion API 的模型
- **可视化模型管理** - 通过拖拽添加、编辑、删除和重新排序自定义模型
- **API 模型发现** - 直接从服务商 API 获取可用模型列表
- **退出保护** - 有未保存更改时关闭会提示警告
- **跨平台** - 支持 macOS、Windows 和 Linux

## 配置说明

DroidGear 读写 `~/.factory/settings.json` 文件：

```json
{
  "customModels": [
    {
      "model": "your-model-id",
      "displayName": "我的自定义模型",
      "baseUrl": "https://api.provider.com/v1",
      "apiKey": "YOUR_API_KEY",
      "provider": "generic-chat-completion-api",
      "maxOutputTokens": 16384
    }
  ]
}
```

### 支持的服务商

| 服务商    | 值                            |
| --------- | ----------------------------- |
| Anthropic | `anthropic`                   |
| OpenAI    | `openai`                      |
| 通用 API  | `generic-chat-completion-api` |

## 开发指南

### 前置要求

- Node.js 20+
- Rust（最新稳定版）
- 平台特定依赖：https://tauri.app/start/prerequisites/

### 启动开发

```bash
npm install
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

## 技术栈

- **前端**: React 19, TypeScript, Vite, Tailwind CSS, shadcn/ui
- **后端**: Tauri v2, Rust
- **状态管理**: Zustand

## 隐私声明

DroidGear 重视您的隐私安全。您的用户名、密码、API 密钥等敏感数据仅存储在本地设备，不会上传至任何服务器。

## 更新日志

### v0.1.7

**新功能**

- 支持 Droid 会话

### v0.1.6

**新功能**

- 复制规格完整路径
- 自动设置显示名称

### v0.1.5

**新功能**

- 添加跳过登录时 websearch 工具调用问题的提示
- 添加环境变量冲突提示
- 检查更新时使用 toast 代替确认框

**问题修复**

- 禁止使用系统模型名称
- 为 Droid 显示名称添加更多特殊字符支持
- 修复切换规格时滚动到顶部
- 修复 Windows 下的错误命令
- 禁用自动更正、自动完成、自动大写和拼写检查

### v0.1.4

**新功能**

- 添加 OpenCode 支持，用于 AI 开发
- 加载和保存 OpenCode 服务商/认证到配置文件
- 点击规格标题可重命名
- OpenCode 服务商使用可调整大小的对话框

**问题修复**

- 修复窗口大小和标签宽度问题
- 修复规格渲染导致窗口溢出
- 修复尝试跳过登录行为变更

### v0.1.3

**新功能**

- 支持规格选择和编辑模式
- 添加每小时检查更新
- 更新器添加发布说明显示
- 支持 new-api OpenAI 模型检测

**问题修复**

- 切换服务商类型时不清空 API URL

### v0.1.2

**新功能**

- 添加 MCP 预设
- MCP 服务器管理
- 添加更多 Droid 设置选项
- 禁止在别名/显示名称中使用括号
- 自动生成 ID 和索引，与 Factory Droid 规则一致

**问题修复**

- 修复 sub2api OpenAI 模型不再追加 /v1
- 修复默认填充 maxOutputTokens

### v0.1.1

**新功能**

- 云端会话同步开关
- 添加安装说明区域
- 确保显示名称唯一性

**问题修复**

- 修复默认标签页为 droid

### v0.1.0

**新功能**

- 添加关于页面，移除示例
- 支持 sub2api 的 antigravity 平台

**问题修复**

- 修复样式 lint 问题
- 修复文本居中对齐
- 修复获取所有密钥分页问题

### v0.0.9

**新功能**

- 添加使用 /model 切换模型的提示
- 支持设置默认模型并标记当前默认
- 增强对话框拖拽功能

**问题修复**

- 修复获取密钥的问题
- 修复频道保存路径错误（不影响 Droid 设置）
- 修复切换 Droid 标签页时丢失更改的问题
- 修复渲染代码块时的深色模式颜色问题

### v0.0.8

**新功能**

- Specs 面板支持重命名、删除、另存为操作

**问题修复**

- 禁用全局右键菜单

### v0.0.7

**新功能**

- 新增 Specs 面板功能，支持查看 ~/.factory/specs 目录下的规格文件
- 支持 Markdown 格式渲染规格文件内容

### v0.0.6

**新功能**

- 新增跳过登录辅助功能

**问题修复**

- 修复二进制上传问题
- 修复 Anthropic 模型获取：第三方代理服务现支持 OpenAI 风格的 Bearer token 认证

### v0.0.5

**新功能**

- 复制模型功能
- 筛选和批量删除功能
- 导入导出配置功能
- 新增 sub2api 平台支持
- 从频道添加模型时支持选择服务商

**问题修复**

- 修复 codex/gemini 平台支持（gemini 现仅支持 v1beta/models）
- 修复无平台时 API 路径问题
- 修复导入时模型滚动问题

### v0.0.4

**新功能**

- 版本检查和升级提示
- 默认禁用快捷面板

**问题修复**

- 修复模型 key 唯一性问题
- 修复界面溢出布局问题
- 修复切换时未保存确认提示
- 修复频道名称映射问题

## 致谢

本项目基于 [tauri-template](https://github.com/dannysmith/tauri-template) 开发，感谢 Danny Smith 提供的优秀模板。

## 许可证

[MIT](LICENSE.md)
