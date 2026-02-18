# DroidGear

[中文](README.md)

A desktop enhancement tool for [Factory Droid](https://factory.ai) / [OpenClaw](https://openclaw.ai) / [Codex](https://github.com/openai/codex) / [OpenCode](https://opencode.ai).

Supports custom AI models (BYOK), embedded terminal, session & specs management, MCP server configuration, and more.

## Installation

### macOS

Downloaded apps may be blocked by Gatekeeper since they are not signed by Apple. Run this command to fix:

```bash
xattr -cr /Applications/DroidGear.app
```

### Windows / Linux

Run the installer directly.

## Features

### Custom Model Management

- **Multi-Provider Support** - Configure models from Anthropic, OpenAI, or any Generic Chat Completion API
- **Visual Model Management** - Add, edit, delete, and reorder custom models with drag-and-drop
- **Batch Operations** - Copy models, filter and batch delete, set default model
- **API Model Discovery** - Fetch available models directly from provider APIs with auto-generated IDs and display names
- **Import/Export** - Support configuration import/export and batch management
- **Multi-Platform Support** - Support sub2api, antigravity and other API proxy platforms

### Embedded Terminal

- **Built-in Terminal** - Integrated terminal with state save and restore
- **Custom Configuration** - Custom font, force dark mode
- **Convenient Operations** - Copy-on-select, OSC 9 notifications, derived sub-windows
- **Keyboard Shortcuts** - Cmd/Ctrl+1~0 to switch terminals, Cmd/Ctrl+W to close tabs, Cmd/Ctrl+Shift+[ to switch tabs
- **Code Snippets** - Snippets support on terminal pages

### Droid Session Management

- **Session Viewer** - View and manage Droid sessions with delete support
- **Multiple Views** - Toggle between list/grouped view, hide empty sessions
- **Follow Mode** - Session follow mode with thinking expansion toggle
- **Cloud Sync** - Cloud session sync toggle

### Specs File Management

- **File Browser** - View spec files in `~/.factory/specs` directory
- **Markdown Rendering** - Support Markdown format rendering
- **File Operations** - Rename, delete, save as, copy full path
- **Edit Mode** - Support spec selection and edit mode

### MCP Server Management

- **Presets** - Built-in MCP presets (including exa, etc.)
- **Server Management** - MCP server configuration management

### OpenCode Support

- **AI Development Integration** - OpenCode tool integration
- **Configuration Management** - Load and save provider/auth configurations

### OpenClaw Support

- **AI Development Integration** - OpenClaw tool integration
- **Configuration Management** - Load and save provider configurations
- **Installation Helper** - Built-in installation commands for macOS/Linux/Windows

### Codex Support

- **Codex CLI Integration** - Manage Codex configuration profiles
- **Configuration Management** - Load and save auth/config.toml (`~/.codex`)
- **Management Pages** - MCP servers / sessions / terminal subpages under Codex

### Other Features

- **Skip Login Helper** - Helper for skipping login flow
- **Auto Update** - Version check, auto update notification and download progress
- **Window State** - Save and restore window state
- **Exit Protection** - Warns before closing with unsaved changes
- **Cross-Platform** - Works on macOS, Windows, and Linux

## Configuration

DroidGear reads and writes to `~/.factory/settings.json`:

```json
{
  "customModels": [
    {
      "model": "your-model-id",
      "displayName": "My Custom Model",
      "baseUrl": "https://api.provider.com/v1",
      "apiKey": "YOUR_API_KEY",
      "provider": "generic-chat-completion-api",
      "maxOutputTokens": 16384
    }
  ]
}
```

### Supported Providers

| Provider    | Value                         |
| ----------- | ----------------------------- |
| Anthropic   | `anthropic`                   |
| OpenAI      | `openai`                      |
| Generic API | `generic-chat-completion-api` |

## Development

### Prerequisites

- Node.js 20+
- Rust (latest stable)
- Platform-specific dependencies: https://tauri.app/start/prerequisites/

### Setup

```bash
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## Tech Stack

- **Frontend**: React 19, TypeScript, Vite, Tailwind CSS, shadcn/ui
- **Backend**: Tauri v2, Rust
- **State**: Zustand

## Privacy

DroidGear values your privacy. Your username, password, API keys, and other sensitive data are stored locally on your device only and are never uploaded to any server.

## Changelog

### v0.3.8

**New Features**

- Add OpenAI Responses message type support for OpenClaw

**Bug Fixes**

- Use bash instead of sh for OpenCode install command
- Fix ugly close button
- Reduce white splash screen
- Reduce UI jump glitch
- Fix CSS build warning

See full changelog at [CHANGELOG.md](CHANGELOG.md)

## Acknowledgements

This project is based on [tauri-template](https://github.com/dannysmith/tauri-template) by Danny Smith. Thanks for the excellent template!

## License

[MIT](LICENSE.md)
