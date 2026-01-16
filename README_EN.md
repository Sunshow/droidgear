# DroidGear

[中文](README.md)

A desktop application for configuring custom AI models in [Factory Droid](https://factory.ai) using BYOK (Bring Your Own Key).

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
- **API Model Discovery** - Fetch available models directly from provider APIs
- **Import/Export** - Support configuration import/export and batch management
- **Multi-Platform Support** - Support sub2api, antigravity and other API proxy platforms

### Embedded Terminal

- **Built-in Terminal** - Integrated terminal with state save and restore
- **Custom Configuration** - Custom font, force dark mode
- **Convenient Operations** - Copy-on-select, OSC 9 notifications, derived sub-windows

### Droid Session Management

- **Session Viewer** - View and manage Droid sessions
- **Multiple Views** - Toggle between list/grouped view, hide empty sessions
- **Follow Mode** - Session follow mode with thinking expansion toggle
- **Cloud Sync** - Cloud session sync toggle

### Specs File Management

- **File Browser** - View spec files in `~/.factory/specs` directory
- **Markdown Rendering** - Support Markdown format rendering
- **File Operations** - Rename, delete, save as, copy full path

### MCP Server Management

- **Presets** - Built-in MCP presets
- **Server Management** - MCP server configuration management

### OpenCode Support

- **AI Development Integration** - OpenCode tool integration
- **Configuration Management** - Load and save provider/auth configurations

### Other Features

- **Skip Login Helper** - Helper for skipping login flow
- **Auto Update** - Version check and auto update notification
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

### v0.2.4

**New Features**

- Support Cmd/Ctrl+1/2/3..0 to switch terminal
- Delete session
- Update check improvements and downloading progress
- Use custom ActionButton and ActionDropdownMenuItem
- Allow Windows user specify custom terminal command

**Bug Fixes**

- Ensure locale is set for proper CJK character display
- Wrong selection active if IME active
- IME compatibility

### v0.2.3

**New Features**

- Ctrl/Cmd + W to close terminal tab
- Show toggle left sidebar button
- Add Snippets on Terminal pages
- Default use directory name as Terminal name

**Bug Fixes**

- Ctrl/Cmd+W only bind in Terminal page
- Allow empty derived terminal
- Big performance improvement for terminal loading
- Update status checking
- Auto generate display name
- Rename terminal on Windows
- Use windows custom config for github actions

### v0.2.2

**Bug Fixes**

- Fix TERM/COLORTERM environment variable injection

### v0.2.1

**New Features**

- Terminal support open derived sub window
- Support shift+enter on macOS and ctrl+shift+c/v on Windows/Linux

**Bug Fixes**

- Auto focus and selection while rename Terminal name
- Allow use dot in model name
- Terminal bottom style and model maxTokens step size
- Save and restore window state
- Remove custom env to inherit system environment

### v0.2.0

**New Features**

- Terminal support OSC 9 notifications
- Use single button to toggle session list/grouped view
- Sessions hide empty groups
- Use different indicators for Terminal active and notification
- Terminal add copy-on-select
- Fix left panel width

### v0.1.9

**New Features**

- Terminal add reload and remove state control
- Custom terminal font
- Sessions support hiding empty
- Terminal inject envs and add force dark mode

### v0.1.8

**New Features**

- Save and restore terminal status
- Embedded terminals support
- More powerful session follow mode and toggle thinking expansion

### v0.1.7

**New Features**

- Support droid sessions

### v0.1.6

**New Features**

- Copy spec full path
- Auto set displayName

### v0.1.5

**New Features**

- Add tips about websearch tool-call issue if skip login
- Add environment variable conflict hinter
- Use toast instead of confirm while check update

**Bug Fixes**

- Prevent the use of system model names
- Add more special chars for droid display name
- Fix scroll top if switch spec
- Fix wrong command in windows
- Disable autoCorrect autoComplete autoCapitalize spellCheck

### v0.1.4

**New Features**

- Add OpenCode support for AI development
- Load and save OpenCode providers/auth to profiles
- Click Spec title to rename
- Use resizable dialog for OpenCode provider

**Bug Fixes**

- Fix window size and tab width issues
- Fix spec render causing window overflow
- Fix try skip login behavior changed

### v0.1.3

**New Features**

- Support spec selection and edit mode
- Add hourly check update
- Updater add release notes display
- Support new-api OpenAI models detection

**Bug Fixes**

- Do not empty API URL if switch provider type

### v0.1.2

**New Features**

- Add MCP presets
- MCP servers management
- Add more settings of droid
- Prohibit use brackets in alias/displayName
- Auto generate id and index, same rule with factory droid

**Bug Fixes**

- Does not append /v1 for sub2api openai models
- Default fill maxOutputTokens

### v0.1.1

**New Features**

- Cloud session sync toggle
- Add install instruction area
- Ensure unique display name

**Bug Fixes**

- Fix default tab: droid

### v0.1.0

**New Features**

- Add about page, remove examples
- Support antigravity platform of sub2api

**Bug Fixes**

- Fix style lint
- Fix text align center
- Fix fetch all pages of keys

### v0.0.9

**New Features**

- Add tips to use /model to switch model
- Support setting default model and mark current default
- Enhanced dialog dragging functionality

**Bug Fixes**

- Fix fetch keys bug
- Fix wrong channels saved path (does not affect Droid settings)
- Fix losing changes while switching Droid tabs
- Fix dark color issue while rendering code block

### v0.0.8

**New Features**

- Specs panel supports rename, delete, and save as operations

**Bug Fixes**

- Disable global context menu

### v0.0.7

**New Features**

- Added Specs panel to view spec files from ~/.factory/specs directory
- Support Markdown rendering for spec file content

### v0.0.6

**New Features**

- Added skip login helper

**Bug Fixes**

- Fixed uploadPlainBinary issue
- Fixed Anthropic model fetching: now supports OpenAI-style Bearer token auth for third-party proxy services

### v0.0.5

**New Features**

- Copy model functionality
- Filter and batch delete functionality
- Import/export configuration
- Added sub2api platform support
- Provider selection when adding models from channels

**Bug Fixes**

- Fixed codex/gemini platform support (gemini now only supports v1beta/models)
- Fixed API path issue when no platform is set
- Fixed model scrolling issue during import

### v0.0.4

**New Features**

- Version check and upgrade notification
- Quick panel disabled by default

**Bug Fixes**

- Fixed model key uniqueness issue
- Fixed overflow layout issue
- Fixed confirm save prompt when switching with unsaved changes
- Fixed channel name mapping

## Acknowledgements

This project is based on [tauri-template](https://github.com/dannysmith/tauri-template) by Danny Smith. Thanks for the excellent template!

## License

[MIT](LICENSE.md)
