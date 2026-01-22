# DroidGear

[English](README_EN.md)

[Factory Droid](https://factory.ai) / [Codex](https://github.com/openai/codex) / [OpenCode](https://opencode.ai) 的桌面增强管理工具。

支持自定义 AI 模型（BYOK）、嵌入式终端、会话与 Specs 管理、MCP 服务器配置等。

## 安装说明

### macOS

由于应用未经 Apple 签名，下载后可能被 Gatekeeper 阻止运行。请执行以下命令解除限制：

```bash
xattr -cr /Applications/DroidGear.app
```

### Windows / Linux

直接运行安装程序即可。

## 功能特性

### 自定义模型管理

- **多服务商支持** - 配置来自 Anthropic、OpenAI 或任何通用 Chat Completion API 的模型
- **可视化模型管理** - 通过拖拽添加、编辑、删除和重新排序自定义模型
- **批量操作** - 复制模型、筛选和批量删除、设置默认模型
- **API 模型发现** - 直接从服务商 API 获取可用模型列表，自动生成 ID 和显示名称
- **配置导入导出** - 支持配置的导入导出和批量管理
- **多平台支持** - 支持 sub2api、antigravity 等多种 API 代理平台

### 嵌入式终端

- **内置终端** - 集成终端支持，可保存和恢复终端状态
- **自定义配置** - 支持自定义字体、强制深色模式
- **便捷操作** - 选中即复制、OSC 9 通知、派生子窗口
- **快捷键支持** - Cmd/Ctrl+1~0 切换终端、Cmd/Ctrl+W 关闭标签页、Cmd/Ctrl+Shift+[ 切换标签
- **代码片段** - 终端页面支持代码片段功能

### Droid 会话管理

- **会话查看** - 查看和管理 Droid 会话，支持删除会话
- **多种视图** - 列表/分组视图切换，支持隐藏空会话
- **跟随模式** - 会话跟随模式，支持切换思考展开状态
- **云端同步** - 云端会话同步开关

### Specs 规格文件管理

- **文件浏览** - 查看 `~/.factory/specs` 目录下的规格文件
- **Markdown 渲染** - 支持 Markdown 格式渲染
- **文件操作** - 重命名、删除、另存为、复制完整路径
- **编辑模式** - 支持规格选择和编辑模式

### MCP 服务器管理

- **预设配置** - 内置 MCP 预设（包含 exa 等）
- **服务器管理** - MCP 服务器配置管理

### OpenCode 支持

- **AI 开发集成** - OpenCode 工具集成
- **配置管理** - 服务商/认证配置的加载和保存

### Codex 支持

- **Codex CLI 集成** - Codex 配置 Profile 管理
- **配置管理** - 认证与 `config.toml` 的加载和保存（`~/.codex`）
- **管理入口** - 在 Codex 入口下提供 MCP 服务器 / 会话 / 终端 管理子页

### 其他功能

- **跳过登录辅助** - 辅助跳过登录流程
- **自动更新** - 版本检查、自动更新提示和下载进度显示
- **窗口状态** - 保存和恢复窗口状态
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

### v0.2.8

- 支持 WSL 自定义配置路径

### v0.2.7

- Codex CLI 配置支持
- [sub2api] 显示远程分组名称

### v0.2.6

- 通用多模型组件，支持 BYOK 和频道
- 添加新的预设 MCP 服务器 exa 替代 droid websearch
- 修复自动刷新 saveModels 操作

### v0.2.5

- Cmd/Ctrl + Shift + [ 切换到上一个标签页
- 修复删除会话后列表不刷新的问题

查看完整更新日志请访问 [CHANGELOG.md](CHANGELOG.md)

## 致谢

本项目基于 [tauri-template](https://github.com/dannysmith/tauri-template) 开发，感谢 Danny Smith 提供的优秀模板。

感谢 [@xb0or](https://github.com/xb0or) 贡献 Codex CLI 配置支持功能。

## 许可证

[MIT](LICENSE.md)
