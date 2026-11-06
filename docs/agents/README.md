# Agent Coding Guidelines

Detailed coding guidelines for AI agents working on this codebase. These docs are referenced from `AGENTS.md` via progressive disclosure — read them when working on the relevant area.

## Index

| Document                                                  | When to Read                                    |
| --------------------------------------------------------- | ----------------------------------------------- |
| [Commands](./commands.md)                                 | Running builds, lints, tests, or any npm task   |
| [TypeScript/React Code Style](./code-style-typescript.md) | Writing or reviewing TypeScript/React code      |
| [Rust Code Style](./code-style-rust.md)                   | Writing or reviewing Rust code                  |
| [State Management](./state-management.md)                 | Working with Zustand, TanStack Query, or state  |
| [Architecture](./architecture.md)                         | Understanding system design, Tauri bridge, i18n |
| [UI Components](./ui-components.md)                       | Working with Radix UI, shadcn/ui, dialogs       |
| [Upstream Config Sources](./tool-config-sources.md)       | Touching codex / pi / hermes / omp config I/O   |

## Relationship to Other Docs

- **`AGENTS.md`** (project root) — Core rules that always apply. Start here.
- **`docs/developer/`** — Deep-dive technical documentation for specific systems. Agent docs reference these where relevant.
