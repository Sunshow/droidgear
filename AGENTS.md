# AI Agent Instructions

## Overview

Tauri v2 + React 19 desktop app. Uses npm (NOT pnpm), TypeScript strict mode, Zustand for state, TanStack Query for persistence.

## Build/Lint/Test Commands

```bash
# Development
npm run dev              # Start Vite dev server (frontend only)
npm run tauri:dev        # Start full Tauri app with hot reload

# Build
npm run build            # TypeScript check + Vite build
npm run tauri:build      # Full Tauri production build

# Quality Gates (run after significant changes)
npm run check:all        # All checks: typecheck, lint, ast-grep, format, rust checks, tests

# Individual Checks
npm run typecheck        # TypeScript type checking
npm run lint             # ESLint (strict, zero warnings allowed)
npm run lint:fix         # ESLint with auto-fix
npm run format           # Prettier format all files
npm run format:check     # Check formatting without changes
npm run ast:lint         # ast-grep architecture rules
npm run ast:fix          # ast-grep with auto-fix

# Testing - Frontend (Vitest)
npm run test             # Watch mode
npm run test:run         # Single run
npm run test:run -- src/hooks/use-platform.test.ts           # Single file
npm run test:run -- -t "should detect macOS"                 # Single test by name
npm run test:run -- src/store/ui-store.test.ts -t "toggle"   # File + test name
npm run test:coverage    # With coverage report

# Testing - Rust
npm run rust:test        # Run all Rust tests
npm run rust:fmt         # Format Rust code
npm run rust:fmt:check   # Check Rust formatting
npm run rust:clippy      # Rust linter (warnings = errors)
npm run rust:bindings    # Regenerate tauri-specta TypeScript bindings
```

## Code Style Guidelines

### TypeScript/React

**Imports** - Use type imports, path aliases, group by external/internal:

```typescript
import { type ReactNode } from 'react' // Type imports with 'type' keyword
import { useTranslation } from 'react-i18next' // External packages first
import { useUIStore } from '@/store/ui-store' // Use @/ alias for src/
import { logger } from '@/lib/logger' // Internal modules
```

**Formatting** (Prettier enforced):

- No semicolons, single quotes, 2-space indent, 80 char line width
- Trailing commas in ES5 positions, arrow parens avoided when possible

**Naming**:

- Components: `PascalCase` (e.g., `MainWindow.tsx`)
- Hooks: `use-kebab-case.ts` (e.g., `use-platform.ts`)
- Stores: `kebab-case-store.ts` (e.g., `ui-store.ts`)
- Types: `PascalCase`, prefix with `type` import
- Unused vars: prefix with `_` (e.g., `_unusedParam`)

**Error Handling**:

```typescript
// Tauri commands return Result type
const result = await commands.loadPreferences()
if (result.status === 'ok') {
  return result.data
} else {
  logger.error('Failed to load preferences', result.error)
}
```

### Zustand Pattern (CRITICAL - enforced by ast-grep)

```typescript
// ✅ GOOD: Selector syntax prevents render cascades
const leftSidebarVisible = useUIStore(state => state.leftSidebarVisible)

// ❌ BAD: Destructuring causes unnecessary re-renders
const { leftSidebarVisible } = useUIStore()

// ✅ GOOD: Use getState() in callbacks
const handleAction = () => {
  const { setData } = useStore.getState()
  setData(newData)
}
```

### Rust Style

- Edition 2021, MSRV 1.82
- Modern formatting: `format!("{variable}")` not `format!("{}", variable)`
- All warnings treated as errors via clippy
- Use `tauri-specta` for type-safe command bindings

### Radix UI Focus Management

When using Radix UI components (Dialog, DropdownMenu, Popover, etc.), be aware of automatic focus behavior:

- **Problem**: Radix components return focus to the trigger element when closed, which may override manual `focus()` calls
- **Solution**: Use `onCloseAutoFocus` to prevent default behavior and manually control focus:

```tsx
<DropdownMenuContent
  onCloseAutoFocus={e => {
    e.preventDefault()
    targetRef.current?.focus()
  }}
>
```

This applies to: `DialogContent`, `DropdownMenuContent`, `PopoverContent`, `AlertDialogContent`, etc.

## Architecture Patterns

### State Management Onion

```
useState (component) → Zustand (global UI) → TanStack Query (persistent data)
```

### Event-Driven Bridge

- **Rust → React**: `app.emit("event-name", data)` → `listen("event-name", handler)`
- **React → Rust**: Use typed commands from `@/lib/tauri-bindings`

### Tauri Commands (tauri-specta)

```typescript
import { commands } from '@/lib/tauri-bindings'
const result = await commands.loadPreferences() // Type-safe
// NOT: await invoke('load_preferences')         // No type safety
```

### i18n

```typescript
import { useTranslation } from 'react-i18next'
const { t } = useTranslation()
return <h1>{t('myFeature.title')}</h1>
```

Translations in `/locales/*.json`. Use CSS logical properties for RTL.

## Core Rules

1. **Use npm only** - NOT pnpm
2. **Read before editing** - Understand context first
3. **Run `npm run check:all`** after significant changes
4. **No manual memoization** - React Compiler handles it
5. **Tauri v2 docs only** - v1 patterns are incompatible
6. **No unsolicited commits** - Only when explicitly requested
7. **Use `rm -f`** when removing files

## File Organization

```
src/
├── components/          # React components by feature
│   ├── ui/              # shadcn/ui components (don't modify)
│   └── layout/          # Layout components
├── hooks/               # Custom React hooks (use-*.ts)
├── lib/
│   ├── commands/        # Command system
│   └── tauri-bindings.ts # Auto-generated (don't edit)
├── store/               # Zustand stores (*-store.ts)
└── services/            # TanStack Query + Tauri integration
src-tauri/
├── src/commands/        # Rust Tauri commands
└── capabilities/        # Window permissions (security)
locales/                 # i18n translation files
docs/developer/          # Architecture documentation
```

## Version Requirements

Tauri v2.x, React 19.x, Zustand v5.x, Tailwind v4.x, shadcn/ui v4.x, Vite v7.x, Vitest v4.x

## Documentation

See `docs/developer/README.md` for full index. Key docs:

- `architecture-guide.md` - Mental models, security
- `state-management.md` - Zustand patterns
- `tauri-commands.md` - Adding Rust commands
- `static-analysis.md` - Linting tools
