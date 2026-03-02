# 实现OpenClaw 模型 failover 配置支持

## Overview

Add failover model configuration support to the OpenClaw integration in DroidGear. When the primary model fails, OpenClaw can try a list of fallback models in order.

## Implementation

### Rust backend (`src-tauri/src/commands/openclaw.rs`)

- Added `failover_models: Option<Vec<String>>` field to `OpenClawProfile` struct
- Updated `build_openclaw_config()` to write the `failover` array inside `agents.defaults.model`
- Updated `parse_openclaw_config()` to return a 3-tuple including failover models
- Updated all callers of `parse_openclaw_config()`

### TypeScript bindings (`src/lib/bindings.ts`)

- Regenerated via `npm run rust:bindings`
- `OpenClawProfile` now includes `failoverModels?: string[] | null`

### Zustand store (`src/store/openclaw-store.ts`)

- Added `updateFailoverModels: (models: string[]) => Promise<void>` action
- Fixed `createProfile` to include `failoverModels: null`

### i18n

- Added 6 new keys to `locales/en.json` and `locales/zh.json`

### UI (`src/components/openclaw/OpenClawConfigPage.tsx`)

- Added Failover Models section with a Select dropdown and ordered list with remove buttons
