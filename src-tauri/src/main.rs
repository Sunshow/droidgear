// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    if droidgear_core::claude_runtime::matches_internal_launcher_args(&raw_args) {
        if let Err(error) = droidgear_core::claude_runtime::run_internal_launcher_from_env() {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }

    droidgear_lib::run()
}
