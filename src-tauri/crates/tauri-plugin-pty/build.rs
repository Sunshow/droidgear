use std::env;
use std::fs;
use std::path::PathBuf;

const COMMANDS: &[&str] = &["spawn", "write", "read", "resize", "kill", "exitstatus"];

const COMMON_CONTROLS_MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    type="win32"
    name="tauri-plugin-pty.test"
    version="1.0.0.0"
    processorArchitecture="*"
  />
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
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();

    // Windows test binaries link tauri/tao, whose static imports include
    // comctl32's TaskDialogIndirect. That export exists only in
    // Common-Controls v6 (WinSxS), which the loader activates only for
    // manifests requesting it; without one the exe resolves System32's
    // legacy v5 comctl32 and dies at startup with STATUS_ENTRYPOINT_NOT_FOUND
    // (0xC0000139). The app itself gets this manifest from tauri-build, so
    // only the targets linked here (unit tests) need it. The generic
    // `rustc-link-arg` is used because `rustc-link-arg-tests` covers declared
    // `[[test]]` targets only, not the lib test harness.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        let manifest = PathBuf::from(env::var("OUT_DIR").unwrap()).join("test.manifest");
        fs::write(&manifest, COMMON_CONTROLS_MANIFEST).unwrap();
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
    }
}
