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

## 许可证

[MIT](LICENSE.md)
