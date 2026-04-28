use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::path::{Path, PathBuf};

// ============================================================================
// Types (compatible with existing Tauri bindings)
// ============================================================================

/// User-defined configuration paths (only stores explicitly set paths)
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPaths {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencode_auth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openclaw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hermes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pi: Option<String>,
}

/// Effective path info with default indicator
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EffectivePath {
    pub key: String,
    pub path: String,
    pub is_default: bool,
}

/// All effective paths
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EffectivePaths {
    pub factory: EffectivePath,
    pub opencode: EffectivePath,
    pub opencode_auth: EffectivePath,
    pub codex: EffectivePath,
    pub openclaw: EffectivePath,
    pub hermes: EffectivePath,
    pub pi: EffectivePath,
}

/// WSL distribution info
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WslDistro {
    pub name: String,
    pub is_default: bool,
    pub version: u8,
    pub state: String,
}

/// WSL information including distributions and current user
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WslInfo {
    pub available: bool,
    pub distros: Vec<WslDistro>,
}

// ============================================================================
// Core path resolution
// ============================================================================

const SETTINGS_FILE: &str = "settings.json";

fn get_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

fn droidgear_dir_from_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".droidgear")
}

pub fn get_droidgear_settings_path() -> Result<PathBuf, String> {
    Ok(droidgear_dir_from_home(&get_home_dir()?).join(SETTINGS_FILE))
}

pub fn get_droidgear_settings_path_for_home(home_dir: &Path) -> PathBuf {
    droidgear_dir_from_home(home_dir).join(SETTINGS_FILE)
}

fn read_droidgear_settings_from_path(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read settings: {e}"))?;
    if content.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {e}"))
}

fn write_droidgear_settings_to_path(path: &Path, settings: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create .droidgear directory: {e}"))?;
        }
    }

    let temp_path = path.with_extension("tmp");
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    std::fs::write(&temp_path, content).map_err(|e| format!("Failed to write settings: {e}"))?;
    std::fs::rename(&temp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize settings: {e}")
    })?;
    Ok(())
}

fn load_config_paths_from_settings(settings: &Value) -> ConfigPaths {
    settings
        .get("configPaths")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

pub fn load_config_paths() -> ConfigPaths {
    match get_home_dir() {
        Ok(home) => load_config_paths_for_home(&home),
        Err(_) => ConfigPaths::default(),
    }
}

pub fn load_config_paths_for_home(home_dir: &Path) -> ConfigPaths {
    let path = get_droidgear_settings_path_for_home(home_dir);
    match read_droidgear_settings_from_path(&path) {
        Ok(settings) => load_config_paths_from_settings(&settings),
        Err(_) => ConfigPaths::default(),
    }
}

pub fn get_default_paths_for_home(home_dir: &Path) -> Result<EffectivePaths, String> {
    Ok(EffectivePaths {
        factory: EffectivePath {
            key: "factory".to_string(),
            path: default_factory_home_for_home(home_dir)?
                .to_string_lossy()
                .to_string(),
            is_default: true,
        },
        opencode: EffectivePath {
            key: "opencode".to_string(),
            path: default_opencode_config_dir_for_home(home_dir)?
                .to_string_lossy()
                .to_string(),
            is_default: true,
        },
        opencode_auth: EffectivePath {
            key: "opencodeAuth".to_string(),
            path: default_opencode_auth_dir_for_home(home_dir)?
                .to_string_lossy()
                .to_string(),
            is_default: true,
        },
        codex: EffectivePath {
            key: "codex".to_string(),
            path: default_codex_home_for_home(home_dir)?
                .to_string_lossy()
                .to_string(),
            is_default: true,
        },
        openclaw: EffectivePath {
            key: "openclaw".to_string(),
            path: default_openclaw_home_for_home(home_dir)?
                .to_string_lossy()
                .to_string(),
            is_default: true,
        },
        hermes: EffectivePath {
            key: "hermes".to_string(),
            path: default_hermes_home_for_home(home_dir)?
                .to_string_lossy()
                .to_string(),
            is_default: true,
        },
        pi: EffectivePath {
            key: "pi".to_string(),
            path: default_pi_home_for_home(home_dir)?
                .to_string_lossy()
                .to_string(),
            is_default: true,
        },
    })
}

pub fn get_effective_paths_for_home(home_dir: &Path) -> Result<EffectivePaths, String> {
    let config = load_config_paths_for_home(home_dir);

    let factory_path = get_factory_home_for_home(home_dir, &config)?;
    let opencode_path = get_opencode_config_dir_for_home(home_dir, &config)?;
    let opencode_auth_path = get_opencode_auth_dir_for_home(home_dir, &config)?;
    let codex_path = get_codex_home_for_home(home_dir, &config)?;
    let openclaw_path = get_openclaw_home_for_home(home_dir, &config)?;
    let hermes_path = get_hermes_home_for_home(home_dir, &config)?;
    let pi_path = get_pi_home_for_home(home_dir, &config)?;

    Ok(EffectivePaths {
        factory: EffectivePath {
            key: "factory".to_string(),
            path: factory_path.to_string_lossy().to_string(),
            is_default: config.factory.is_none(),
        },
        opencode: EffectivePath {
            key: "opencode".to_string(),
            path: opencode_path.to_string_lossy().to_string(),
            is_default: config.opencode.is_none(),
        },
        opencode_auth: EffectivePath {
            key: "opencodeAuth".to_string(),
            path: opencode_auth_path.to_string_lossy().to_string(),
            is_default: config.opencode_auth.is_none(),
        },
        codex: EffectivePath {
            key: "codex".to_string(),
            path: codex_path.to_string_lossy().to_string(),
            is_default: config.codex.is_none(),
        },
        openclaw: EffectivePath {
            key: "openclaw".to_string(),
            path: openclaw_path.to_string_lossy().to_string(),
            is_default: config.openclaw.is_none(),
        },
        hermes: EffectivePath {
            key: "hermes".to_string(),
            path: hermes_path.to_string_lossy().to_string(),
            is_default: config.hermes.is_none(),
        },
        pi: EffectivePath {
            key: "pi".to_string(),
            path: pi_path.to_string_lossy().to_string(),
            is_default: config.pi.is_none(),
        },
    })
}

pub fn get_effective_paths() -> Result<EffectivePaths, String> {
    let home = get_home_dir()?;
    let mut paths = get_effective_paths_for_home(&home)?;
    // On Windows, use WSL-aware default for Hermes when no custom path is set
    if paths.hermes.is_default {
        paths.hermes.path = default_hermes_home_with_wsl(&home)?
            .to_string_lossy()
            .to_string();
    }
    Ok(paths)
}

pub fn get_default_paths() -> Result<EffectivePaths, String> {
    let home = get_home_dir()?;
    let mut paths = get_default_paths_for_home(&home)?;
    // On Windows, show WSL path as the default for Hermes
    paths.hermes.path = default_hermes_home_with_wsl(&home)?
        .to_string_lossy()
        .to_string();
    Ok(paths)
}

pub fn save_config_path(key: &str, path: &str) -> Result<(), String> {
    save_config_path_for_home(&get_home_dir()?, key, path)
}

pub fn save_config_path_for_home(home_dir: &Path, key: &str, path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    let settings_path = get_droidgear_settings_path_for_home(home_dir);
    let mut settings = read_droidgear_settings_from_path(&settings_path)?;

    let config_paths = settings
        .as_object_mut()
        .ok_or("Invalid settings format")?
        .entry("configPaths")
        .or_insert_with(|| serde_json::json!({}));

    let obj = config_paths
        .as_object_mut()
        .ok_or("Invalid configPaths format")?;

    let storage_key = match key {
        "factory" => "factory",
        "opencode" => "opencode",
        "opencodeAuth" => "opencodeAuth",
        "codex" => "codex",
        "openclaw" => "openclaw",
        "hermes" => "hermes",
        "pi" => "pi",
        _ => return Err(format!("Unknown config path key: {key}")),
    };

    obj.insert(storage_key.to_string(), serde_json::json!(path));
    write_droidgear_settings_to_path(&settings_path, &settings)?;
    Ok(())
}

pub fn reset_config_path(key: &str) -> Result<(), String> {
    reset_config_path_for_home(&get_home_dir()?, key)
}

pub fn reset_config_path_for_home(home_dir: &Path, key: &str) -> Result<(), String> {
    let settings_path = get_droidgear_settings_path_for_home(home_dir);
    let mut settings = read_droidgear_settings_from_path(&settings_path)?;

    if let Some(obj) = settings.as_object_mut() {
        if let Some(config_paths) = obj.get_mut("configPaths") {
            if let Some(paths_obj) = config_paths.as_object_mut() {
                let storage_key = match key {
                    "factory" => "factory",
                    "opencode" => "opencode",
                    "opencodeAuth" => "opencodeAuth",
                    "codex" => "codex",
                    "openclaw" => "openclaw",
                    "hermes" => "hermes",
                    "pi" => "pi",
                    _ => return Err(format!("Unknown config path key: {key}")),
                };
                paths_obj.remove(storage_key);

                if paths_obj.is_empty() {
                    obj.remove("configPaths");
                }
            }
        }
    }

    write_droidgear_settings_to_path(&settings_path, &settings)?;
    Ok(())
}

// ============================================================================
// Default path getters
// ============================================================================

fn default_factory_home_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(home_dir.join(".factory"))
}

fn default_opencode_config_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(home_dir.join(".config").join("opencode"))
}

fn default_opencode_auth_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(home_dir.join(".local").join("share").join("opencode"))
}

fn default_codex_home_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(home_dir.join(".codex"))
}

fn default_openclaw_home_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(home_dir.join(".openclaw"))
}

fn default_hermes_home_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(home_dir.join(".hermes"))
}

/// Pi home defaults to `~/.pi/agent`
fn default_pi_home_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(home_dir.join(".pi").join("agent"))
}

/// On Windows, try to resolve Hermes default path via WSL since Hermes
/// doesn't support native Windows. Falls back to the local home directory.
fn default_hermes_home_with_wsl(home_dir: &Path) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(wsl_info) = get_wsl_info() {
            if let Some(distro) = wsl_info.distros.iter().find(|d| d.is_default) {
                if let Ok(username) = get_wsl_username(&distro.name) {
                    if let Ok(wsl_path) = build_wsl_path(&distro.name, &username, "hermes") {
                        return Ok(PathBuf::from(wsl_path));
                    }
                }
            }
        }
    }
    default_hermes_home_for_home(home_dir)
}

// ============================================================================
// Public path getters (honor overrides)
// ============================================================================

pub fn get_factory_home() -> Result<PathBuf, String> {
    let home = get_home_dir()?;
    let config = load_config_paths_for_home(&home);
    get_factory_home_for_home(&home, &config)
}

pub fn get_factory_home_for_home(home_dir: &Path, config: &ConfigPaths) -> Result<PathBuf, String> {
    match &config.factory {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_factory_home_for_home(home_dir),
    }
}

pub fn get_opencode_config_dir() -> Result<PathBuf, String> {
    let home = get_home_dir()?;
    let config = load_config_paths_for_home(&home);
    get_opencode_config_dir_for_home(&home, &config)
}

pub fn get_opencode_config_dir_for_home(
    home_dir: &Path,
    config: &ConfigPaths,
) -> Result<PathBuf, String> {
    match &config.opencode {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_opencode_config_dir_for_home(home_dir),
    }
}

pub fn get_opencode_auth_dir() -> Result<PathBuf, String> {
    let home = get_home_dir()?;
    let config = load_config_paths_for_home(&home);
    get_opencode_auth_dir_for_home(&home, &config)
}

pub fn get_opencode_auth_dir_for_home(
    home_dir: &Path,
    config: &ConfigPaths,
) -> Result<PathBuf, String> {
    match &config.opencode_auth {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_opencode_auth_dir_for_home(home_dir),
    }
}

pub fn get_codex_home() -> Result<PathBuf, String> {
    let home = get_home_dir()?;
    let config = load_config_paths_for_home(&home);
    get_codex_home_for_home(&home, &config)
}

pub fn get_codex_home_for_home(home_dir: &Path, config: &ConfigPaths) -> Result<PathBuf, String> {
    match &config.codex {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_codex_home_for_home(home_dir),
    }
}

pub fn get_openclaw_home() -> Result<PathBuf, String> {
    let home = get_home_dir()?;
    let config = load_config_paths_for_home(&home);
    get_openclaw_home_for_home(&home, &config)
}

pub fn get_openclaw_home_for_home(
    home_dir: &Path,
    config: &ConfigPaths,
) -> Result<PathBuf, String> {
    match &config.openclaw {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_openclaw_home_for_home(home_dir),
    }
}

pub fn get_hermes_home() -> Result<PathBuf, String> {
    let home = get_home_dir()?;
    let config = load_config_paths_for_home(&home);
    match &config.hermes {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_hermes_home_with_wsl(&home),
    }
}

pub fn get_hermes_home_for_home(home_dir: &Path, config: &ConfigPaths) -> Result<PathBuf, String> {
    match &config.hermes {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_hermes_home_for_home(home_dir),
    }
}

pub fn get_pi_home() -> Result<PathBuf, String> {
    let home = get_home_dir()?;
    let config = load_config_paths_for_home(&home);
    get_pi_home_for_home(&home, &config)
}

pub fn get_pi_home_for_home(home_dir: &Path, config: &ConfigPaths) -> Result<PathBuf, String> {
    match &config.pi {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_pi_home_for_home(home_dir),
    }
}

// ============================================================================
// WSL (Windows only)
// ============================================================================

#[cfg(not(target_os = "windows"))]
pub fn get_wsl_info() -> Result<WslInfo, String> {
    Ok(WslInfo {
        available: false,
        distros: vec![],
    })
}

#[cfg(target_os = "windows")]
pub fn get_wsl_info() -> Result<WslInfo, String> {
    use std::process::Command;

    let output = Command::new("wsl").args(["-l", "-v"]).output();
    match output {
        Ok(output) => {
            if !output.status.success() {
                return Ok(WslInfo {
                    available: false,
                    distros: vec![],
                });
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let distros = parse_wsl_list(&stdout);

            Ok(WslInfo {
                available: !distros.is_empty(),
                distros,
            })
        }
        Err(_e) => Ok(WslInfo {
            available: false,
            distros: vec![],
            // keep error out of UI - matches previous behavior
        }),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_wsl_username(_distro: &str) -> Result<String, String> {
    Err("WSL is only available on Windows".to_string())
}

#[cfg(target_os = "windows")]
pub fn get_wsl_username(distro: &str) -> Result<String, String> {
    use std::process::Command;

    let output = Command::new("wsl")
        .args(["-d", distro, "whoami"])
        .output()
        .map_err(|e| format!("Failed to run wsl whoami: {e}"))?;

    if !output.status.success() {
        return Err("Failed to get WSL username".to_string());
    }

    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if username.is_empty() {
        return Err("Empty username returned from WSL".to_string());
    }
    Ok(username)
}

pub fn build_wsl_path(distro: &str, username: &str, config_key: &str) -> Result<String, String> {
    let subdir = match config_key {
        "factory" => ".factory",
        "opencode" => ".config/opencode",
        "opencodeAuth" => ".local/share/opencode",
        "codex" => ".codex",
        "openclaw" => ".openclaw",
        "hermes" => ".hermes",
        "pi" => ".pi/agent",
        _ => return Err(format!("Unknown config key: {config_key}")),
    };

    Ok(format!(r"\\wsl$\{}\home\{}\{}", distro, username, subdir))
}

#[cfg(target_os = "windows")]
fn parse_wsl_list(output: &str) -> Vec<WslDistro> {
    let mut distros = Vec::new();

    for line in output.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let line: String = line
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '*' || *c == '-')
            .collect();

        let is_default = line.starts_with('*');
        let line = line.trim_start_matches('*').trim();

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let version: u8 = parts.last().and_then(|v| v.parse().ok()).unwrap_or(2);
            let state = parts[parts.len() - 2].to_string();
            let name = parts[..parts.len() - 2].join(" ");

            if !name.is_empty() {
                distros.push(WslDistro {
                    name,
                    is_default,
                    version,
                    state,
                });
            }
        }
    }

    distros
}
