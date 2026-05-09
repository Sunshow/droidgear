# DroidGear

[中文](README.md)

A desktop enhancement tool for [Factory Droid](https://factory.ai) / [OpenClaw](https://openclaw.ai) / [Codex](https://github.com/openai/codex) / [OpenCode](https://opencode.ai) / [Pi](https://pi.dev).

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
- **Model Registry** - Browse and search available AI models from a built-in registry in Preferences
- **API Model Discovery** - Fetch available models directly from provider APIs with auto-generated IDs and display names
- **Import/Export** - Support configuration import/export and batch management
- **Multi-Platform Support** - Support sub2api, antigravity and other API proxy platforms
- **Ollama Support** - Ollama channel support with automatic local service detection
- **Provider Templates** - Built-in OpenAI, Gemini provider templates for quick setup

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

### Droid Configuration Management

- **Multi-Settings Files** - Manage and switch between multiple Factory Droid configuration files (settings.json)
- **Terminal Preferences** - Set terminal preferences independently for each configuration file
- **Panel Refresh** - Automatically refresh relevant panels when switching configuration files

### Missions Management

- **Model Configuration** - Configure Worker model and Validation Worker model for Missions
- **Reasoning Effort** - Set reasoning effort independently for each model (none/low/medium/high/max), displays 1M context badge on model cards

### Specs File Management

- **File Browser** - View spec files in `~/.factory/specs` directory
- **Markdown Rendering** - Support Markdown format rendering
- **File Operations** - Rename, delete, save as, copy full path
- **Edit Mode** - Support spec selection and edit mode
- **Export** - Export spec items

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

### Hermes Agent Support

- **Configuration Management** - Hermes Agent YAML profile management
- **Channel Import** - Import Hermes Agent configuration from channels

### Pi Support

- **Pi Coding Agent Integration** - Pi (pi.dev) custom model configuration management
- **Provider/Model Management** - Support configuring multiple providers and their models (baseUrl, api, apiKey, headers, compat, etc.)
- **Profile Management** - Multi-profile support with one-click apply to `~/.pi/agent/models.json`
- **Live Config Reading** - Load existing configuration from Pi's live config

### Other Features

- **Auto Update** - Version check, auto update notification and download progress
- **Window State** - Save and restore window state
- **Exit Protection** - Warns before closing with unsaved changes
- **Cross-Platform** - Works on macOS, Windows, and Linux

## TUI Version (Headless Terminal)

DroidGear TUI is a terminal interface version designed for SSH and headless environments, sharing the same configuration files and core functionality with the desktop version.

### Installation

Download the `droidgear-tui` binary for your platform from [Releases](https://github.com/Sunshow/droidgear/releases):

- macOS (Apple Silicon): `droidgear-tui-*-aarch64-apple-darwin.tar.gz`
- macOS (Intel): `droidgear-tui-*-x86_64-apple-darwin.tar.gz`
- Linux: `droidgear-tui-*-x86_64-unknown-linux-gnu.tar.gz`
- Windows: `droidgear-tui-*-x86_64-pc-windows-msvc.zip`
- Windows: `droidgear-tui-*-x86_64-pc-windows-msvc.zip`

Extract and place the binary in your PATH (e.g., `/usr/local/bin`).

### Running

```bash
# Use default configuration (reads from ~/.factory and ~/.droidgear)
droidgear-tui

# Specify custom HOME path (for containers/testing)
droidgear-tui --home /path/to/custom/home

# Run a Codex profile once (hands off execution to the current terminal)
droidgear-tui run codex <profile-id>
```

### Supported Features

The TUI version supports the following configuration management features:

- **Factory Configuration**: Custom model management, default model settings
- **MCP Servers**: CRUD operations, enable/disable, import/export
- **Codex Profiles**: Configuration file management, change preview, one-click apply
- **OpenCode Profiles**: Provider/Auth configuration management
- **OpenClaw Profiles**: Configuration management and apply
- **Pi Profiles**: Provider/Model configuration management and apply
- **Sessions**: Session browsing and management
- **Paths**: Path override configuration (for server environments)
- **Channels**: Proxy platform and credential management

### Basic Operations

- `↑/↓` or `j/k`: Move up/down
- `Enter`: Enter/Confirm
- `Esc`: Back/Cancel
- `Tab`: Switch focus area
- `Ctrl+S`: Preview changes (edit page)
- `y/N`: Confirm apply changes
- `q`: Quit (main screen)

### Configuration Files

The TUI version shares configuration files with the desktop version:

- Factory config: `~/.factory/settings.json`
- MCP config: `~/.factory/mcp.json`
- DroidGear config: `~/.droidgear/`
- Codex config: `~/.codex/`
- OpenCode config: `~/.config/opencode/`
- OpenClaw config: `~/.openclaw/`
- Pi config: `~/.pi/agent/`

For detailed design documentation, see [docs/developer/tui-design.md](docs/developer/tui-design.md)

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

See full changelog at [CHANGELOG.md](CHANGELOG.md)

## Acknowledgements

This project is based on [tauri-template](https://github.com/dannysmith/tauri-template) by Danny Smith. Thanks for the excellent template!

## License

[MIT](LICENSE.md)
