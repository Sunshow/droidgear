# Standalone TUI cross-build workflow (spike).
# Isolated from the main tauri/npm pipeline: artifacts land in dist/tui/
# (gitignored) and no existing npm script or CI workflow is touched.

default:
    @just --list

# Native Windows build (MSVC, host toolchain)
tui-windows:
    cd src-tauri && cargo build -p droidgear-tui --release
    mkdir -p dist/tui/x86_64-pc-windows-msvc
    cp src-tauri/target/release/droidgear-tui.exe dist/tui/x86_64-pc-windows-msvc/

# Static musl build via cargo-zigbuild (no Linux machine needed)
tui-linux:
    cd src-tauri && cargo zigbuild -p droidgear-tui --target x86_64-unknown-linux-musl --release
    mkdir -p dist/tui/x86_64-unknown-linux-musl
    cp src-tauri/target/x86_64-unknown-linux-musl/release/droidgear-tui dist/tui/x86_64-unknown-linux-musl/

# Everything buildable from the current host (macOS excluded: needs Apple SDK)
tui-all: tui-windows tui-linux
    @echo "--- built ---"
    @ls -lh dist/tui/*/droidgear-tui*

# macOS cross-compilation is not possible from Windows (Apple SDK frameworks
# are required at link time). Build on a macOS runner instead:
#   cargo build -p droidgear-tui --release --target aarch64-apple-darwin
tui-macos:
    @echo "error: macOS TUI binaries must be built on a macOS host/runner (Apple SDK required)."
    @echo "See .github/workflows or run this on a macOS machine."
    exit 1

# Remove cross-build artifacts (does NOT touch src-tauri/target caches)
tui-clean:
    rm -rf dist/tui
