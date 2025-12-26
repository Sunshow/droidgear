# Claude Code Instructions

Read @AGENTS.md for all project instructions.

## Release Procedure

To release a new version, run:

```bash
npm run release:prepare <version>
```

Example:

```bash
npm run release:prepare v0.0.3
```

The script will:

1. Check git status is clean
2. Run `npm run check:all`
3. Update version in package.json, tauri.conf.json, Cargo.toml
4. Update lock files
5. Prompt to execute git commands (commit, tag, push)

After push, GitHub Actions will build and create a draft release. You need to manually publish the draft release on GitHub.

## Local Status

@CLAUDE.local.md
