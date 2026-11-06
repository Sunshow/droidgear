use std::env;
use std::fs;
use std::path::PathBuf;

/// The application manifest, embedded into **every** linked executable of this
/// package (bins *and* test harnesses).
///
/// It is the same content `tauri_build` installs by default (a dependency on
/// Common-Controls v6), because Tauri's Windows bindings statically import
/// `comctl32!TaskDialogIndirect`, which only exists in Common-Controls v6
/// (WinSxS). The loader activates that version only for a binary whose manifest
/// asks for it; without one the exe binds System32's legacy v5 `comctl32.dll`
/// and dies before `main` with `STATUS_ENTRYPOINT_NOT_FOUND` (0xC0000139).
///
/// `tauri_build` puts its manifest in a resource library linked to `[[bin]]`
/// targets only, so the unit-test harness got nothing and `cargo test` (hence
/// `npm run rust:bindings`) could not start at all. The manifest is therefore
/// owned here and passed as a linker argument, which applies to every target;
/// `tauri_build` is told to skip its own copy so the two do not collide
/// (`LNK1123: duplicate resource ... type:MANIFEST`).
const APP_MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#;

fn main() {
    let attributes = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    tauri_build::try_build(attributes).expect("failed to run tauri-build");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is always set by cargo"));
        let manifest = out_dir.join("app.manifest");
        fs::write(&manifest, APP_MANIFEST).expect("write the app manifest");
        // `/MANIFEST:EMBED` is required by link.exe for any /MANIFESTINPUT
        // (LNK1220); it generates the (single) manifest resource itself.
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
    }
}
