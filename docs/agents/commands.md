# Build/Lint/Test Commands

Complete reference for all project commands.

## Development

```bash
npm run dev              # Start Vite dev server (frontend only)
npm run tauri:dev        # Start full Tauri app with hot reload
```

## Build

```bash
npm run build            # TypeScript check + Vite build
npm run tauri:build      # Full Tauri production build
```

## Quality Gates

```bash
# Run after significant changes (REQUIRED)
npm run check:all        # All checks: typecheck, lint, ast-grep, format, rust checks, tests
```

## Individual Checks

```bash
npm run typecheck        # TypeScript type checking
npm run lint             # ESLint (strict, zero warnings allowed)
npm run lint:fix         # ESLint with auto-fix
npm run format           # Prettier format all files
npm run format:check     # Check formatting without changes
npm run ast:lint         # ast-grep architecture rules
npm run ast:fix          # ast-grep with auto-fix
```

## Testing — Frontend (Vitest)

```bash
npm run test             # Watch mode
npm run test:run         # Single run
npm run test:run -- src/hooks/use-platform.test.ts           # Single file
npm run test:run -- -t "should detect macOS"                 # Single test by name
npm run test:run -- src/store/ui-store.test.ts -t "toggle"   # File + test name
npm run test:coverage    # With coverage report
```

## TUI (Terminal UI)

```bash
npm run tui              # Run TUI (cargo run)
npm run tui:dev          # Run TUI with auto-rebuild via cargo-watch
npm run tui:build        # Compile TUI in release mode
npm run tui:check        # Check TUI compiles without producing binary
```

## TUI Cross-Builds (just + zig)

Standalone TUI binary for other platforms, without leaving Windows. Artifacts
go to `dist/tui/<target>/` (gitignored) — isolated from the main tauri/npm
pipeline. Prerequisites: `cargo-zigbuild` + `zig` (scoop). `sccache` is
optional (faster rebuilds only) — see below.

```bash
just                     # List available recipes
just tui-linux           # Static musl single binary (x86_64-unknown-linux-musl)
just tui-windows         # Native MSVC build
just tui-all             # Everything buildable from current host
just tui-clean           # Remove dist/tui/ artifacts only
```

❌ BAD: `cargo build --target x86_64-unknown-linux-musl` directly — C deps
(rusqlite/zstd) need a C cross-toolchain. `cargo zigbuild` uses zig as the C
compiler/linker, so no Linux toolchain is needed.

macOS targets cannot be cross-compiled from Windows (Apple SDK frameworks are
required at link time) — use a macOS runner instead. See
[cross-platform.md](../developer/cross-platform.md).

### Optional: sccache

`sccache` only speeds up rebuilds; it is NOT required. The repo must not
contain a cargo config that hard-requires it — configure it locally instead
via the gitignored `.cargo/config.toml` (or user-level
`~/.cargo/config.toml`):

```toml
[build]
rustc-wrapper = "sccache"
```

## Testing — Rust

```bash
npm run rust:test        # Run all Rust tests
npm run rust:fmt         # Format Rust code
npm run rust:fmt:check   # Check Rust formatting
npm run rust:clippy      # Rust linter (warnings = errors)
npm run rust:bindings    # Regenerate tauri-specta TypeScript bindings

## All Commands (Quick Reference)

| Category | Command | Description |
|----------|---------|-------------|
| Dev | `npm run dev` | Vite frontend dev server |
| Dev | `npm run tauri:dev` | Tauri app with hot reload |
| Dev | `npm run tui:dev` | TUI with auto-rebuild |
| Build | `npm run build` | TypeScript check + Vite build |
| Build | `npm run tauri:build` | Tauri production build |
| Build | `npm run tui:build` | TUI release build |
| Build | `just tui-linux` | TUI static musl binary via zig |
| Check | `npm run tui:check` | TUI compile check |
| Run | `npm run tui` | Run TUI |
| Quality | `npm run check:all` | Full check suite |
| Quality | `npm run fix:all` | Auto-fix all fixable issues |
| Test | `npm run test:run` | Vitest (single run) |
| Test | `npm run rust:test` | Rust tests |
```
