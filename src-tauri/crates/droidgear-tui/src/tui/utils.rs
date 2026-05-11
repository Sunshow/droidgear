use super::*;

pub(super) fn write_string(path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

#[derive(Debug, Default, serde::Deserialize)]
pub(super) struct StoredPreferences {
    #[serde(default)]
    droid_run: Option<droidgear_core::droid_runtime::DroidRunPreferences>,
}

pub(super) fn preferences_path() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join(APP_IDENTIFIER).join("preferences.json"))
}

pub(super) fn load_droid_run_preferences_from_path(
    path: &Path,
) -> anyhow::Result<droidgear_core::droid_runtime::DroidRunPreferences> {
    if !path.exists() {
        return Ok(droidgear_core::droid_runtime::DroidRunPreferences::default());
    }

    let contents =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let prefs: StoredPreferences =
        serde_json::from_str(&contents).with_context(|| format!("parse {}", path.display()))?;
    Ok(prefs.droid_run.unwrap_or_default())
}

pub(super) fn load_droid_run_preferences(
) -> anyhow::Result<droidgear_core::droid_runtime::DroidRunPreferences> {
    let Some(path) = preferences_path() else {
        return Ok(droidgear_core::droid_runtime::DroidRunPreferences::default());
    };

    load_droid_run_preferences_from_path(&path)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn format_string_list(values: &[String], empty_label: &str) -> String {
    if values.is_empty() {
        return format!("  {empty_label}\n");
    }

    let mut out = String::new();
    for value in values {
        out.push_str("  - ");
        out.push_str(value);
        out.push('\n');
    }
    out
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn format_env_pairs(values: &[(String, String)], empty_label: &str) -> String {
    if values.is_empty() {
        return format!("  {empty_label}\n");
    }

    let mut out = String::new();
    for (key, value) in values {
        out.push_str("  - ");
        out.push_str(key);
        out.push('=');
        out.push_str(value);
        out.push('\n');
    }
    out
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn format_optional_value(value: Option<&str>, empty_label: &str) -> String {
    match value {
        Some(value) => format!("  {value}\n"),
        None => format!("  {empty_label}\n"),
    }
}

pub(super) fn start_command_in_foreground(
    program: &str,
    args: &[String],
    env: &[(String, String)],
    secret_env: &[(String, String)],
    unset_env: &[String],
    cwd: Option<&Path>,
) -> anyhow::Result<()> {
    let mut command = Command::new(program);
    command.args(args);

    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    for key in unset_env {
        command.env_remove(key);
    }

    for (key, value) in env.iter().chain(secret_env.iter()) {
        command.env(key, value);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let error = command.exec();
        Err(error).with_context(|| format!("exec {program}"))
    }

    #[cfg(not(unix))]
    {
        command
            .spawn()
            .with_context(|| format!("start {program}"))?;
        Ok(())
    }
}

pub(super) fn sanitize_terminal_for_direct_exec() -> anyhow::Result<()> {
    use std::io::IsTerminal;

    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Ok(());
    }

    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    stdout
        .write_all(
            b"\x1b[0m\x1b[?25h\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1004l\x1b[?1006l\x1b[?2004l",
        )
        .context("write terminal reset sequence")?;
    stdout.flush().context("flush terminal reset sequence")?;

    #[cfg(unix)]
    unsafe {
        if libc::tcflush(libc::STDIN_FILENO, libc::TCIFLUSH) != 0 {
            return Err(std::io::Error::last_os_error()).context("flush terminal input queue");
        }
    }

    Ok(())
}

fn probe_claude_cli() -> anyhow::Result<()> {
    let version_output = Command::new("claude")
        .arg("--version")
        .output()
        .with_context(|| "Failed to execute claude --version".to_string())?;
    if !version_output.status.success() {
        anyhow::bail!("Failed to read Claude CLI version");
    }

    Ok(())
}

fn current_launcher_program() -> anyhow::Result<String> {
    std::env::current_exe()
        .map(|path| path.to_string_lossy().to_string())
        .with_context(|| "locate current launcher executable".to_string())
}

pub(super) fn format_diff_report(
    title: &str,
    files: Vec<(String, Option<String>, Option<String>)>,
) -> String {
    let mut out = String::new();
    out.push_str(title);
    out.push_str("\n\n");

    let mut any = false;
    for (label, before, after) in files {
        if before.as_deref() == after.as_deref() {
            continue;
        }
        any = true;
        out.push_str(&format!("=== {label} ===\n"));

        let before_s = before.as_deref().unwrap_or("");
        let after_s = after.as_deref().unwrap_or("");
        let diff = TextDiff::from_lines(before_s, after_s);
        out.push_str(
            &diff
                .unified_diff()
                .header(&format!("{label} (before)"), &format!("{label} (after)"))
                .to_string(),
        );
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }

    if !any {
        out.push_str("No changes.\n");
    }

    out
}

pub(super) fn build_droid_temporary_run_plan(
    home_dir: &Path,
    settings_path: &Path,
) -> anyhow::Result<droidgear_core::droid_runtime::DroidTemporaryRunPlan> {
    let prefs = load_droid_run_preferences()?;
    droidgear_core::droid_runtime::cleanup_stale_temp_settings_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::droid_runtime::build_temporary_run_plan_from_settings_path_for_home(
        home_dir,
        settings_path,
        &prefs,
    )
    .map_err(anyhow::Error::msg)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn preview_droid_temporary_run(
    home_dir: &Path,
    settings_path: &Path,
) -> anyhow::Result<String> {
    let plan = build_droid_temporary_run_plan(home_dir, settings_path)?;

    let mut out = String::new();
    out.push_str("Droid temporary run preview\n\n");
    out.push_str(&format!(
        "Source settings path:\n  {}\n\n",
        settings_path.display()
    ));
    out.push_str(&format!(
        "Temporary settings path:\n  {}\n\n",
        plan.temp_settings_path.display()
    ));
    out.push_str("Program:\n");
    out.push_str(&format!("  {}\n\n", plan.program));
    out.push_str("Args:\n");
    out.push_str(&format_string_list(&plan.args, "(none)"));
    out.push('\n');
    out.push_str("Environment overrides:\n");
    out.push_str(&format_env_pairs(&plan.env, "(none)"));
    out.push('\n');
    out.push_str("Unset environment variables:\n");
    out.push_str(&format_string_list(&plan.unset_env, "(none)"));

    Ok(out)
}

pub(super) fn run_droid_temporary_run(home_dir: &Path, settings_path: &Path) -> anyhow::Result<()> {
    let plan = build_droid_temporary_run_plan(home_dir, settings_path)?;
    start_command_in_foreground(
        &plan.program,
        &plan.args,
        &plan.env,
        &[],
        &plan.unset_env,
        None,
    )
}

pub fn run_droid_temporary_run_for_settings_name(
    home_dir: &Path,
    settings_name: &str,
) -> anyhow::Result<()> {
    sanitize_terminal_for_direct_exec()?;
    let settings_path = droidgear_core::droid_settings_files::get_settings_path_by_name_for_home(
        home_dir,
        settings_name,
    )
    .map_err(anyhow::Error::msg)?;
    run_droid_temporary_run(home_dir, &settings_path)
}

pub fn list_droid_temporary_run_targets(home_dir: &Path) -> anyhow::Result<String> {
    let files = droidgear_core::droid_settings_files::list_settings_files_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;

    let mut out = String::from("Available Droid run targets:\n");
    for file in files {
        let marker = if file.is_active { "*" } else { " " };
        let selector = if file.is_global { "global" } else { &file.name };
        out.push_str(&format!("{marker} {selector}\n"));
    }
    out.push_str("\nUse `droidgear-tui run droid <settings-name>`.\n");
    out.push_str("`*` marks the currently active settings file.");
    Ok(out)
}

pub(super) fn preview_codex_apply(home_dir: &Path, profile_id: &str) -> anyhow::Result<String> {
    let status = droidgear_core::codex::get_codex_config_status_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    let real_config_path = PathBuf::from(status.config_path);
    let real_auth_path = PathBuf::from(status.auth_path);
    let real_active_path = home_dir
        .join(".droidgear")
        .join("codex")
        .join("active-profile.txt");

    let before_config = read_to_string_if_exists(&real_config_path)?;
    let before_auth = read_to_string_if_exists(&real_auth_path)?;
    let before_active = read_to_string_if_exists(&real_active_path)?;

    let temp = TempDir::new().context("create temp home")?;
    let temp_home = temp.path();

    let temp_config_path = temp_home.join(".codex").join("config.toml");
    let temp_auth_path = temp_home.join(".codex").join("auth.json");
    if let Some(ref s) = before_config {
        write_string(&temp_config_path, s)?;
    }
    if let Some(ref s) = before_auth {
        write_string(&temp_auth_path, s)?;
    }

    let profile = droidgear_core::codex::get_codex_profile_for_home(home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::codex::save_codex_profile_for_home(temp_home, profile)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::codex::apply_codex_profile_for_home(temp_home, profile_id)
        .map_err(anyhow::Error::msg)?;

    let after_config = read_to_string_if_exists(&temp_config_path)?;
    let after_auth = read_to_string_if_exists(&temp_auth_path)?;
    let temp_active_path = temp_home
        .join(".droidgear")
        .join("codex")
        .join("active-profile.txt");
    let after_active = read_to_string_if_exists(&temp_active_path)?;

    Ok(format_diff_report(
        "Codex apply preview",
        vec![
            (
                real_config_path.to_string_lossy().to_string(),
                before_config,
                after_config,
            ),
            (
                real_auth_path.to_string_lossy().to_string(),
                before_auth,
                after_auth,
            ),
            (
                real_active_path.to_string_lossy().to_string(),
                before_active,
                after_active,
            ),
        ],
    ))
}

pub(super) fn build_codex_temporary_run_plan(
    home_dir: &Path,
    profile_id: &str,
) -> anyhow::Result<droidgear_core::codex_runtime::CodexTemporaryLaunchPlan> {
    droidgear_core::codex_runtime::cleanup_stale_runtime_homes_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    let profile = droidgear_core::codex::get_codex_profile_for_home(home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::codex_runtime::build_temporary_run_plan_for_home(home_dir, &profile)
        .map_err(anyhow::Error::msg)
}

pub(super) fn run_codex_temporary_run(home_dir: &Path, profile_id: &str) -> anyhow::Result<()> {
    let plan = build_codex_temporary_run_plan(home_dir, profile_id)?;
    start_command_in_foreground(
        &plan.program,
        &plan.args,
        &plan.env,
        &plan.secret_env,
        &plan.unset_env,
        None,
    )
}

pub(super) fn preview_codex_temporary_run(
    home_dir: &Path,
    profile_id: &str,
) -> anyhow::Result<String> {
    let plan = build_codex_temporary_run_plan(home_dir, profile_id)?;

    let mut out = String::new();
    out.push_str("Codex temporary run preview\n\n");
    out.push_str("Runtime CODEX_HOME:\n");
    out.push_str(&format!("  {}\n\n", plan.runtime_home_path.display()));
    out.push_str("Program:\n");
    out.push_str(&format!("  {}\n\n", plan.program));
    out.push_str("Args:\n");
    out.push_str(&format_string_list(&plan.args, "(none)"));
    out.push('\n');
    out.push_str("Environment overrides:\n");
    out.push_str(&format_env_pairs(&plan.env, "(none)"));
    out.push('\n');
    out.push_str("Unset environment variables:\n");
    out.push_str(&format_string_list(&plan.unset_env, "(none)"));
    out.push('\n');
    let secret_env_keys = plan
        .secret_env
        .iter()
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    out.push_str("Secret environment keys:\n");
    out.push_str(&format_string_list(&secret_env_keys, "(none)"));
    out.push('\n');
    out.push_str("Warnings:\n");
    out.push_str(&format_string_list(&plan.warnings, "(none)"));

    Ok(out)
}

pub fn list_codex_temporary_run_targets(home_dir: &Path) -> anyhow::Result<String> {
    let profiles = droidgear_core::codex::list_codex_profiles_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    let active_profile_id = droidgear_core::codex::get_active_codex_profile_id_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;

    let mut out = String::from("Available Codex run targets:\n");
    if profiles.is_empty() {
        out.push_str("(none)\n\nUse the Codex TUI/GUI to create a profile first.");
        return Ok(out);
    }

    for (index, profile) in profiles.iter().enumerate() {
        let marker = if active_profile_id.as_deref() == Some(profile.id.as_str()) {
            "*"
        } else {
            " "
        };
        out.push_str(&format!(
            "{marker} {}. {} [id: {}]\n",
            index + 1,
            profile.name,
            profile.id
        ));
    }
    out.push_str("\nUse `droidgear-tui run codex <index|name|id>`.\n");
    out.push_str("`*` marks the currently active profile.");
    Ok(out)
}

pub fn run_codex_temporary_run_for_selector(home_dir: &Path, selector: &str) -> anyhow::Result<()> {
    sanitize_terminal_for_direct_exec()?;
    let profile =
        droidgear_core::codex::resolve_codex_profile_selector_for_home(home_dir, selector)
            .map_err(anyhow::Error::msg)?;
    run_codex_temporary_run(home_dir, &profile.id)
}

pub(super) fn build_claude_temporary_run_plan(
    home_dir: &Path,
    profile_id: &str,
) -> anyhow::Result<droidgear_core::claude_runtime::ClaudeTemporaryLaunchPlan> {
    droidgear_core::claude_runtime::cleanup_stale_runtime_dirs_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    let profile = droidgear_core::claude::get_claude_profile_for_home(home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    let launcher_program = current_launcher_program()?;
    let launcher_args = droidgear_core::claude_runtime::internal_launcher_args();
    droidgear_core::claude_runtime::build_temporary_run_plan_for_home(
        home_dir,
        &profile,
        &launcher_program,
        &launcher_args,
    )
    .map_err(|e| anyhow::Error::msg(format!("Failed to prepare Claude temporary run: {e}")))
}

pub(super) fn run_claude_temporary_run(home_dir: &Path, profile_id: &str) -> anyhow::Result<()> {
    probe_claude_cli().map_err(|error| {
        let message = error.to_string();
        if message.starts_with("Failed to execute claude --version") {
            anyhow::Error::msg("Claude CLI is not installed or not available in PATH.")
        } else if message == "Failed to read Claude CLI version" {
            anyhow::Error::msg(
                "Failed to inspect the installed Claude CLI. Check that `claude` runs correctly in your shell.",
            )
        } else {
            error
        }
    })?;

    let plan = build_claude_temporary_run_plan(home_dir, profile_id)?;
    for warning in &plan.warnings {
        eprintln!("Warning: {warning}");
    }
    start_command_in_foreground(
        &plan.program,
        &plan.args,
        &plan.env,
        &plan.secret_env,
        &plan.unset_env,
        None,
    )
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn format_claude_temporary_run_preview(
    preview: &droidgear_core::claude_runtime::ClaudeTemporaryRunDebugPreview,
) -> String {
    let mut out = String::new();
    out.push_str("Claude temporary run preview\n\n");
    out.push_str("Sensitive preview:\n");
    out.push_str(
        "  The settings overlay and launcher payload below may contain bearer tokens.\n\n",
    );
    out.push_str("Profile:\n");
    out.push_str(&format!(
        "  {} [id: {}]\n\n",
        preview.profile_name, preview.profile_id
    ));
    out.push_str("Live config dir:\n");
    out.push_str(&format!("  {}\n\n", preview.live_config_dir));
    out.push_str("Inherited CLAUDE_ENV_FILE source:\n");
    out.push_str(&format_optional_value(
        preview.inherited_env_file_source.as_deref(),
        "(none)",
    ));
    out.push('\n');
    out.push_str("Settings overlay JSON:\n");
    out.push_str("  Empty-string values below are intentional tombstones that clear inherited/live Claude env keys.\n\n");
    out.push_str(&preview.settings_overlay_json);
    if !preview.settings_overlay_json.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str("Launcher program:\n");
    out.push_str(&format!("  {}\n\n", preview.program));
    out.push_str("Launcher args:\n");
    out.push_str(&format_string_list(&preview.args, "(none)"));
    out.push('\n');
    out.push_str("Claude child program:\n");
    out.push_str(&format!("  {}\n\n", preview.child_program));
    out.push_str("Claude child args:\n");
    out.push_str(&format_string_list(&preview.child_args, "(none)"));
    out.push('\n');
    out.push_str("Environment overrides:\n");
    out.push_str(&format_env_pairs(&preview.env, "(none)"));
    out.push('\n');
    out.push_str("Unset environment variables:\n");
    out.push_str(&format_string_list(&preview.unset_env, "(none)"));
    out.push('\n');
    out.push_str("Secret launcher env keys:\n");
    out.push_str(&format_string_list(&preview.secret_env_keys, "(none)"));
    out.push('\n');
    out.push_str("Warnings:\n");
    out.push_str(&format_string_list(&preview.warnings, "(none)"));
    out
}

pub(super) fn preview_claude_temporary_run(
    home_dir: &Path,
    profile_id: &str,
) -> anyhow::Result<String> {
    let profile = droidgear_core::claude::get_claude_profile_for_home(home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    let launcher_program = current_launcher_program()?;
    let launcher_args = droidgear_core::claude_runtime::internal_launcher_args();
    let preview = droidgear_core::claude_runtime::build_temporary_run_debug_preview_for_home(
        home_dir,
        &profile,
        &launcher_program,
        &launcher_args,
    )
    .map_err(anyhow::Error::msg)?;
    Ok(format_claude_temporary_run_preview(&preview))
}

pub fn list_claude_temporary_run_targets(home_dir: &Path) -> anyhow::Result<String> {
    let profiles = droidgear_core::claude::list_claude_profiles_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    let active_profile_id = droidgear_core::claude::get_active_claude_profile_id_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;

    let mut out = String::from("Available Claude run targets:\n");
    if profiles.is_empty() {
        out.push_str("(none)\n\nUse the Claude TUI/GUI to create a profile first.");
        return Ok(out);
    }

    for (index, profile) in profiles.iter().enumerate() {
        let marker = if active_profile_id.as_deref() == Some(profile.id.as_str()) {
            "*"
        } else {
            " "
        };
        out.push_str(&format!(
            "{marker} {}. {} [id: {}]\n",
            index + 1,
            profile.name,
            profile.id
        ));
    }
    out.push_str("\nUse `droidgear-tui run claude <index|name|id>`.\n");
    out.push_str(
        "Use `droidgear-tui run claude --preview <index|name|id>` to inspect the launch overlay and internal launcher payload.\n",
    );
    out.push_str("`*` marks the currently active profile.");
    Ok(out)
}

pub fn run_claude_temporary_run_for_selector(
    home_dir: &Path,
    selector: &str,
) -> anyhow::Result<()> {
    // Claude temp runs already exec through the internal launcher, which owns
    // the final terminal reset immediately before `exec claude`.
    let profile =
        droidgear_core::claude::resolve_claude_profile_selector_for_home(home_dir, selector)
            .map_err(anyhow::Error::msg)?;
    run_claude_temporary_run(home_dir, &profile.id)
}

pub fn preview_claude_temporary_run_for_selector(
    home_dir: &Path,
    selector: &str,
) -> anyhow::Result<String> {
    let profile =
        droidgear_core::claude::resolve_claude_profile_selector_for_home(home_dir, selector)
            .map_err(anyhow::Error::msg)?;
    preview_claude_temporary_run(home_dir, &profile.id)
}

pub(super) fn preview_opencode_apply(home_dir: &Path, profile_id: &str) -> anyhow::Result<String> {
    let status = droidgear_core::opencode::get_opencode_config_status_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    let real_config_path = PathBuf::from(status.config_path);
    let real_auth_path = PathBuf::from(status.auth_path);
    let real_active_path = home_dir
        .join(".droidgear")
        .join("opencode")
        .join("active-profile.txt");

    let before_config = read_to_string_if_exists(&real_config_path)?;
    let before_auth = read_to_string_if_exists(&real_auth_path)?;
    let before_active = read_to_string_if_exists(&real_active_path)?;

    let temp = TempDir::new().context("create temp home")?;
    let temp_home = temp.path();

    let config_file_name = real_config_path
        .file_name()
        .unwrap_or(std::ffi::OsStr::new("opencode.json"));
    let auth_file_name = real_auth_path
        .file_name()
        .unwrap_or(std::ffi::OsStr::new("auth.json"));

    let temp_config_path = temp_home
        .join(".config")
        .join("opencode")
        .join(config_file_name);
    let temp_auth_path = temp_home
        .join(".local")
        .join("share")
        .join("opencode")
        .join(auth_file_name);

    if let Some(ref s) = before_config {
        write_string(&temp_config_path, s)?;
    }
    if let Some(ref s) = before_auth {
        write_string(&temp_auth_path, s)?;
    }

    let profile = droidgear_core::opencode::get_opencode_profile_for_home(home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::opencode::save_opencode_profile_for_home(temp_home, profile)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::opencode::apply_opencode_profile_for_home(temp_home, profile_id)
        .map_err(anyhow::Error::msg)?;

    let after_config = read_to_string_if_exists(&temp_config_path)?;
    let after_auth = read_to_string_if_exists(&temp_auth_path)?;
    let temp_active_path = temp_home
        .join(".droidgear")
        .join("opencode")
        .join("active-profile.txt");
    let after_active = read_to_string_if_exists(&temp_active_path)?;

    Ok(format_diff_report(
        "OpenCode apply preview",
        vec![
            (
                real_config_path.to_string_lossy().to_string(),
                before_config,
                after_config,
            ),
            (
                real_auth_path.to_string_lossy().to_string(),
                before_auth,
                after_auth,
            ),
            (
                real_active_path.to_string_lossy().to_string(),
                before_active,
                after_active,
            ),
        ],
    ))
}

pub(super) fn preview_openclaw_apply(home_dir: &Path, profile_id: &str) -> anyhow::Result<String> {
    let status = droidgear_core::openclaw::get_openclaw_config_status_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    let real_config_path = PathBuf::from(status.config_path);
    let real_active_path = home_dir
        .join(".droidgear")
        .join("openclaw")
        .join("active-profile.txt");

    let before_config = read_to_string_if_exists(&real_config_path)?;
    let before_active = read_to_string_if_exists(&real_active_path)?;

    let temp = TempDir::new().context("create temp home")?;
    let temp_home = temp.path();

    let temp_config_path = temp_home.join(".openclaw").join("openclaw.json");
    if let Some(ref s) = before_config {
        write_string(&temp_config_path, s)?;
    }

    let profile = droidgear_core::openclaw::get_openclaw_profile_for_home(home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::openclaw::save_openclaw_profile_for_home(temp_home, profile)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::openclaw::apply_openclaw_profile_for_home(temp_home, profile_id)
        .map_err(anyhow::Error::msg)?;

    let after_config = read_to_string_if_exists(&temp_config_path)?;
    let temp_active_path = temp_home
        .join(".droidgear")
        .join("openclaw")
        .join("active-profile.txt");
    let after_active = read_to_string_if_exists(&temp_active_path)?;

    Ok(format_diff_report(
        "OpenClaw apply preview",
        vec![
            (
                real_config_path.to_string_lossy().to_string(),
                before_config,
                after_config,
            ),
            (
                real_active_path.to_string_lossy().to_string(),
                before_active,
                after_active,
            ),
        ],
    ))
}

pub(super) fn byte_index_for_char(value: &str, char_idx: usize) -> usize {
    value
        .char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(value.len())
}

pub(super) fn remove_char_at(value: &mut String, char_idx: usize) {
    let byte_idx = byte_index_for_char(value, char_idx);
    if byte_idx < value.len() {
        value.remove(byte_idx);
    }
}

pub(super) fn insert_char_at(value: &mut String, char_idx: usize, c: char) {
    let byte_idx = byte_index_for_char(value, char_idx);
    value.insert(byte_idx, c);
}

pub(super) fn factory_model_id(
    model: Option<&droidgear_core::factory_settings::CustomModel>,
    index: usize,
) -> Option<String> {
    let model = model?;
    if let Some(id) = model.id.clone().filter(|s| !s.trim().is_empty()) {
        return Some(id);
    }
    let display = model
        .display_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| model.model.clone());
    Some(format!("custom:{display}-{index}"))
}
