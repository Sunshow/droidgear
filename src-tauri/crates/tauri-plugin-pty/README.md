# DroidGear PTY plugin patch

Based on the MIT-licensed [`tauri-plugin-pty` 0.2.1](https://crates.io/crates/tauri-plugin-pty/0.2.1) by Tnze, upstream commit [`de31beef`](https://github.com/Tnze/tauri-plugin-pty/tree/de31beef7a1964d6da7f80a9d0d1289fbadc8054).

The upstream async commands call blocking terminal reads, writes, and child-process waits directly on Tauri's async workers. Several idle terminals can occupy every worker, leaving window dragging and other async commands waiting indefinitely. Upstream 0.3.1 still has these blocking calls.

This local copy moves `read`, `write`, and `exitstatus` I/O and their mutex acquisition into `tauri::async_runtime::spawn_blocking`. It preserves the 0.2.1 command names, permissions, and response formats used by the `tauri-pty` JavaScript package. Remove this patch when a compatible upstream release provides the fix.

The regression test invokes all three commands with deliberately blocked I/O on a runtime with three workers. An unrelated async request must finish before that I/O is released. The test also checks the commands' return values after release.

From `src-tauri`, run:

```sh
cargo test -p tauri-plugin-pty
```

The plugin is included in `npm run rust:test` and `npm run rust:clippy`.
