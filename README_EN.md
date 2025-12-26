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

- **Multi-Provider Support** - Configure models from Anthropic, OpenAI, or any Generic Chat Completion API
- **Visual Model Management** - Add, edit, delete, and reorder custom models with drag-and-drop
- **API Model Discovery** - Fetch available models directly from provider APIs
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

## License

[MIT](LICENSE.md)
