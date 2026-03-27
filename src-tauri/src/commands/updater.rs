//! Custom updater commands for portable Windows builds.

use std::str::FromStr;

#[cfg(any(target_os = "windows", test))]
use std::path::{Path, PathBuf};
#[cfg(target_os = "windows")]
use std::time::Duration;

#[cfg(target_os = "windows")]
use base64::{engine::general_purpose::STANDARD, Engine};
#[cfg(target_os = "windows")]
use minisign_verify::{PublicKey, Signature};
use reqwest::{Client, StatusCode};
use semver::Version;
use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use sha2::{Digest, Sha256};
use specta::Type;
use tauri::AppHandle;
#[cfg(target_os = "windows")]
use tauri::Manager;
#[cfg(target_os = "windows")]
use uuid::Uuid;

const PORTABLE_MANIFEST_URL: &str =
    "https://github.com/Sunshow/droidgear/releases/latest/download/latest-portable.json";
const RELEASES_TAG_URL: &str = "https://github.com/Sunshow/droidgear/releases/tag/";
#[cfg(any(target_os = "windows", test))]
const PORTABLE_WINDOWS_FILENAME: &str = "droidgear_windows_x64.exe";
#[cfg(target_os = "windows")]
const UPDATE_CHANNEL_MARKER_FILENAME: &str = "update-channel.json";
const UPDATER_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum UpdateChannel {
    Managed,
    Portable,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PortableUpdateInfo {
    pub version: String,
    pub body: Option<String>,
    pub pub_date: Option<String>,
    pub url: String,
    pub signature: String,
    pub sha256: String,
    pub release_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortableUpdateManifest {
    version: String,
    #[serde(default, alias = "body")]
    notes: Option<String>,
    #[serde(default, alias = "pubDate")]
    pub_date: Option<String>,
    url: String,
    signature: String,
    sha256: String,
    #[serde(default, alias = "releaseUrl")]
    release_url: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateChannelMarker {
    exe_path: String,
    channel: UpdateChannel,
}

#[cfg(any(target_os = "windows", test))]
#[derive(Debug, Clone, Default)]
struct WindowsInstallDirs {
    program_files: Option<PathBuf>,
    program_files_x86: Option<PathBuf>,
    local_app_data: Option<PathBuf>,
}

impl PortableUpdateManifest {
    fn into_update_info(self) -> PortableUpdateInfo {
        let version = normalize_version(&self.version);

        PortableUpdateInfo {
            version: version.clone(),
            body: normalize_optional_text(self.notes),
            pub_date: normalize_optional_text(self.pub_date),
            url: self.url,
            signature: self.signature,
            sha256: self.sha256,
            release_url: self
                .release_url
                .filter(|url| !url.trim().is_empty())
                .unwrap_or_else(|| release_url_for_version(&version)),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_update_channel(app: AppHandle) -> Result<UpdateChannel, String> {
    resolve_update_channel(&app)
}

#[tauri::command]
#[specta::specta]
pub async fn check_portable_update(app: AppHandle) -> Result<Option<PortableUpdateInfo>, String> {
    if resolve_update_channel(&app)? != UpdateChannel::Portable {
        return Ok(None);
    }

    let Some(manifest) = fetch_portable_manifest().await? else {
        return Ok(None);
    };

    if !is_newer_version(env!("CARGO_PKG_VERSION"), &manifest.version)? {
        return Ok(None);
    }

    Ok(Some(manifest.into_update_info()))
}

#[tauri::command]
#[specta::specta]
pub async fn install_portable_update(
    app: AppHandle,
    update: PortableUpdateInfo,
) -> Result<(), String> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        let _ = update;
        return Err("Portable updates are only supported on Windows".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        if resolve_update_channel(&app)? != UpdateChannel::Portable {
            return Err("Portable update requested for a managed installation".to_string());
        }

        if !is_newer_version(env!("CARGO_PKG_VERSION"), &update.version)? {
            return Err("No newer portable update is available".to_string());
        }

        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {e}"))?;

        let temp_dir = std::env::temp_dir().join(format!("droidgear-updater-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create updater temp directory: {e}"))?;

        let downloaded_exe_path = temp_dir.join(PORTABLE_WINDOWS_FILENAME);
        let helper_script_path = temp_dir.join("apply-portable-update.ps1");

        let bytes = download_update_bytes(&update.url).await?;
        verify_update_hash(&bytes, &update.sha256)?;
        verify_update_signature(&bytes, &update.signature)?;

        std::fs::write(&downloaded_exe_path, &bytes)
            .map_err(|e| format!("Failed to write downloaded update: {e}"))?;

        write_helper_script(&helper_script_path)?;

        Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-WindowStyle",
                "Hidden",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(&helper_script_path)
            .arg("-CurrentExe")
            .arg(&current_exe)
            .arg("-DownloadedExe")
            .arg(&downloaded_exe_path)
            .spawn()
            .map_err(|e| format!("Failed to launch portable update helper: {e}"))?;

        let app_handle = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(250));
            app_handle.exit(0);
        });

        Ok(())
    }
}

async fn fetch_portable_manifest() -> Result<Option<PortableUpdateManifest>, String> {
    let client = Client::builder()
        .user_agent(UPDATER_USER_AGENT)
        .build()
        .map_err(|e| format!("Failed to build update client: {e}"))?;

    let response = client
        .get(PORTABLE_MANIFEST_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch portable update manifest: {e}"))?;

    match response.status() {
        StatusCode::NO_CONTENT | StatusCode::NOT_FOUND => Ok(None),
        status if status.is_success() => response
            .json::<PortableUpdateManifest>()
            .await
            .map(Some)
            .map_err(|e| format!("Failed to parse portable update manifest: {e}")),
        status => Err(format!(
            "Portable update manifest request failed with status {status}"
        )),
    }
}

#[cfg(target_os = "windows")]
async fn download_update_bytes(url: &str) -> Result<Vec<u8>, String> {
    let client = Client::builder()
        .user_agent(UPDATER_USER_AGENT)
        .build()
        .map_err(|e| format!("Failed to build download client: {e}"))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download portable update: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Portable update download failed with status {}",
            response.status()
        ));
    }

    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|e| format!("Failed to read portable update bytes: {e}"))
}

#[cfg(target_os = "windows")]
fn verify_update_hash(bytes: &[u8], expected_sha256: &str) -> Result<(), String> {
    let actual_sha256 = format!("{:x}", Sha256::digest(bytes));

    if !actual_sha256.eq_ignore_ascii_case(expected_sha256.trim()) {
        return Err("Portable update hash verification failed".to_string());
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn verify_update_signature(bytes: &[u8], signature: &str) -> Result<(), String> {
    let public_key = updater_public_key()?;
    let decoded_public_key = base64_to_string(&public_key)?;
    let decoded_signature = base64_to_string(signature)?;

    let public_key = PublicKey::decode(&decoded_public_key)
        .map_err(|e| format!("Failed to decode updater public key: {e}"))?;
    let signature = Signature::decode(&decoded_signature)
        .map_err(|e| format!("Failed to decode portable update signature: {e}"))?;

    public_key
        .verify(bytes, &signature, true)
        .map_err(|e| format!("Portable update signature verification failed: {e}"))?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn updater_public_key() -> Result<String, String> {
    let config = include_str!("../../tauri.conf.json");
    let json = serde_json::from_str::<serde_json::Value>(config)
        .map_err(|e| format!("Failed to parse embedded Tauri config: {e}"))?;

    json.get("plugins")
        .and_then(|plugins| plugins.get("updater"))
        .and_then(|updater| updater.get("pubkey"))
        .and_then(|pubkey| pubkey.as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| "Updater public key missing from Tauri config".to_string())
}

#[cfg(target_os = "windows")]
fn base64_to_string(value: &str) -> Result<String, String> {
    let decoded = STANDARD
        .decode(value)
        .map_err(|e| format!("Failed to decode base64 value: {e}"))?;

    std::str::from_utf8(&decoded)
        .map(|value| value.to_string())
        .map_err(|e| format!("Decoded base64 value is not valid UTF-8: {e}"))
}

fn resolve_update_channel(app: &AppHandle) -> Result<UpdateChannel, String> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        Ok(UpdateChannel::Managed)
    }

    #[cfg(target_os = "windows")]
    {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {e}"))?;
        let normalized_current_exe = normalize_windows_path(&current_exe);

        if let Some(marker) = read_update_channel_marker(app) {
            if marker.exe_path == normalized_current_exe {
                return Ok(marker.channel);
            }
        }

        let channel = detect_windows_update_channel(&current_exe, &windows_install_dirs());

        if let Err(error) = write_update_channel_marker(
            app,
            &UpdateChannelMarker {
                exe_path: normalized_current_exe,
                channel,
            },
        ) {
            log::warn!("Failed to persist update channel marker: {error}");
        }

        Ok(channel)
    }
}

#[cfg(target_os = "windows")]
fn read_update_channel_marker(app: &AppHandle) -> Option<UpdateChannelMarker> {
    let marker_path = update_channel_marker_path(app).ok()?;
    if !marker_path.exists() {
        return None;
    }

    let contents = std::fs::read_to_string(marker_path).ok()?;
    serde_json::from_str(&contents).ok()
}

#[cfg(target_os = "windows")]
fn write_update_channel_marker(
    app: &AppHandle,
    marker: &UpdateChannelMarker,
) -> Result<(), String> {
    let marker_path = update_channel_marker_path(app)?;
    let temp_path = marker_path.with_extension("tmp");
    let contents = serde_json::to_string_pretty(marker)
        .map_err(|e| format!("Failed to serialize update channel marker: {e}"))?;

    std::fs::write(&temp_path, contents)
        .map_err(|e| format!("Failed to write update channel marker: {e}"))?;
    std::fs::rename(&temp_path, &marker_path)
        .map_err(|e| format!("Failed to finalize update channel marker: {e}"))?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn update_channel_marker_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {e}"))?;

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {e}"))?;

    Ok(app_data_dir.join(UPDATE_CHANNEL_MARKER_FILENAME))
}

#[cfg(any(target_os = "windows", test))]
fn detect_windows_update_channel(
    executable_path: &Path,
    install_dirs: &WindowsInstallDirs,
) -> UpdateChannel {
    if executable_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .is_some_and(|file_name| file_name.eq_ignore_ascii_case(PORTABLE_WINDOWS_FILENAME))
    {
        return UpdateChannel::Portable;
    }

    let executable_path = normalize_windows_path(executable_path);

    if install_dirs
        .program_files
        .as_ref()
        .is_some_and(|dir| executable_path.starts_with(&normalize_windows_path(dir)))
    {
        return UpdateChannel::Managed;
    }

    if install_dirs
        .program_files_x86
        .as_ref()
        .is_some_and(|dir| executable_path.starts_with(&normalize_windows_path(dir)))
    {
        return UpdateChannel::Managed;
    }

    if let Some(local_app_data) = &install_dirs.local_app_data {
        let local_programs = join_windows_path(local_app_data, "Programs");
        if executable_path.starts_with(&local_programs) {
            return UpdateChannel::Managed;
        }
    }

    UpdateChannel::Portable
}

#[cfg(any(target_os = "windows", test))]
fn normalize_windows_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

#[cfg(any(target_os = "windows", test))]
fn join_windows_path(base: &Path, child: &str) -> String {
    let mut path = normalize_windows_path(base);
    if !path.ends_with('\\') {
        path.push('\\');
    }
    path.push_str(child);
    path.to_ascii_lowercase()
}

#[cfg(target_os = "windows")]
fn windows_install_dirs() -> WindowsInstallDirs {
    WindowsInstallDirs {
        program_files: std::env::var_os("ProgramFiles").map(PathBuf::from),
        program_files_x86: std::env::var_os("ProgramFiles(x86)").map(PathBuf::from),
        local_app_data: std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
    }
}

fn normalize_version(version: &str) -> String {
    version.trim().trim_start_matches('v').to_string()
}

fn release_url_for_version(version: &str) -> String {
    format!("{RELEASES_TAG_URL}v{}", normalize_version(version))
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn is_newer_version(current_version: &str, available_version: &str) -> Result<bool, String> {
    let current_version = Version::from_str(&normalize_version(current_version))
        .map_err(|e| format!("Failed to parse current version: {e}"))?;
    let available_version = Version::from_str(&normalize_version(available_version))
        .map_err(|e| format!("Failed to parse available version: {e}"))?;

    Ok(available_version > current_version)
}

#[cfg(target_os = "windows")]
fn write_helper_script(script_path: &Path) -> Result<(), String> {
    let script = r#"
param(
  [Parameter(Mandatory = $true)][string]$CurrentExe,
  [Parameter(Mandatory = $true)][string]$DownloadedExe
)

$ErrorActionPreference = 'Stop'

for ($attempt = 0; $attempt -lt 120; $attempt++) {
  try {
    if (Test-Path -LiteralPath $CurrentExe) {
      Remove-Item -LiteralPath $CurrentExe -Force
    }

    Move-Item -LiteralPath $DownloadedExe -Destination $CurrentExe -Force
    Start-Process -FilePath $CurrentExe
    exit 0
  } catch {
    Start-Sleep -Milliseconds 500
  }
}

exit 1
"#;

    std::fs::write(script_path, script)
        .map_err(|e| format!("Failed to write portable update helper script: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_install_dirs() -> WindowsInstallDirs {
        WindowsInstallDirs {
            program_files: Some(PathBuf::from(r"C:\Program Files")),
            program_files_x86: Some(PathBuf::from(r"C:\Program Files (x86)")),
            local_app_data: Some(PathBuf::from(r"C:\Users\Alice\AppData\Local")),
        }
    }

    #[test]
    fn detects_portable_updates_from_release_filename() {
        let channel = detect_windows_update_channel(
            Path::new(r"C:\Users\Alice\Downloads\droidgear_windows_x64.exe"),
            &sample_install_dirs(),
        );

        assert_eq!(channel, UpdateChannel::Portable);
    }

    #[test]
    fn detects_managed_updates_from_program_files_path() {
        let channel = detect_windows_update_channel(
            Path::new(r"C:\Program Files\DroidGear\droidgear.exe"),
            &sample_install_dirs(),
        );

        assert_eq!(channel, UpdateChannel::Managed);
    }

    #[test]
    fn defaults_unknown_windows_paths_to_portable() {
        let channel = detect_windows_update_channel(
            Path::new(r"D:\Tools\droidgear.exe"),
            &sample_install_dirs(),
        );

        assert_eq!(channel, UpdateChannel::Portable);
    }

    #[test]
    fn compares_versions_with_optional_v_prefix() {
        assert!(is_newer_version("0.5.3", "v0.5.4").expect("version comparison should work"));
        assert!(!is_newer_version("0.5.3", "0.5.3").expect("version comparison should work"));
    }

    #[test]
    fn builds_release_url_with_v_prefix() {
        assert_eq!(
            release_url_for_version("0.5.4"),
            "https://github.com/Sunshow/droidgear/releases/tag/v0.5.4"
        );
        assert_eq!(
            release_url_for_version("v0.5.4"),
            "https://github.com/Sunshow/droidgear/releases/tag/v0.5.4"
        );
    }
}
