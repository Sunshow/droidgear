//! Playground media commands shared by the video and image pages.
//!
//! Saving a generated video or image is done entirely on the Rust side: the
//! save dialog picks the destination and the file is copied from the
//! playground media directory, or downloaded when the provider only returned a
//! URL. Keeping it here means the frontend needs neither the fs nor the opener
//! plugin scope to hand a generated file back to the user.

use std::path::PathBuf;
use std::time::Duration;

use droidgear_core::paths;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// Upper bound for downloading a provider URL the playground never cached.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

/// Where the bytes to save come from.
enum MediaSource {
    /// A file the playground already downloaded into its media directory.
    Local(PathBuf),
    /// A provider URL the playground never cached, e.g. New API's `url`.
    Remote(String),
}

impl MediaSource {
    fn parse(source_path: Option<String>, source_url: Option<String>) -> Result<Self, String> {
        if let Some(path) = normalise(source_path) {
            return validate_local_path(&path).map(Self::Local);
        }
        if let Some(url) = normalise(source_url) {
            return validate_remote_url(&url).map(Self::Remote);
        }
        Err("This task has no media file to save yet.".to_string())
    }

    /// Extension of the source, used when the suggested name carries none.
    fn extension(&self) -> Option<String> {
        match self {
            Self::Local(path) => path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(extension_of),
            Self::Remote(url) => {
                let path = url.split(['?', '#']).next().unwrap_or(url);
                extension_of(path.rsplit('/').next().unwrap_or(path))
            }
        }
    }
}

fn normalise(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

/// Playground media is always written into the playground media directory, so
/// the command refuses to copy anything else out of the user's disk.
fn validate_local_path(raw: &str) -> Result<PathBuf, String> {
    let path = std::fs::canonicalize(raw)
        .map_err(|error| format!("The media file is missing: {error}"))?;
    if !path.is_file() {
        return Err("The media path is not a file.".to_string());
    }
    let home = paths::get_home_dir()?;
    let media_dir = paths::droidgear_dir_from_home(&home)
        .join("playground")
        .join("media");
    let media_dir = std::fs::canonicalize(&media_dir)
        .map_err(|error| format!("The playground media directory is missing: {error}"))?;
    if !path.starts_with(&media_dir) {
        return Err("The media file is outside the playground directory.".to_string());
    }
    Ok(path)
}

fn validate_remote_url(raw: &str) -> Result<String, String> {
    let url = reqwest::Url::parse(raw).map_err(|error| format!("Invalid media URL: {error}"))?;
    match url.scheme() {
        "http" | "https" => Ok(url.to_string()),
        scheme => Err(format!("Unsupported media URL scheme: {scheme}")),
    }
}

/// Keeps the dialog's default name a plain file name: no directory traversal
/// and no hidden dot file.
fn sanitize_file_name(raw: &str) -> String {
    let name = raw.rsplit(['/', '\\']).next().unwrap_or(raw).trim();
    let name = name.trim_start_matches('.');
    if name.is_empty() {
        "playground-media".to_string()
    } else {
        name.to_string()
    }
}

fn extension_of(name: &str) -> Option<String> {
    let (_, extension) = name.rsplit_once('.')?;
    let extension = extension.trim().to_ascii_lowercase();
    let valid = !extension.is_empty()
        && extension.len() <= 8
        && extension.chars().all(|c| c.is_ascii_alphanumeric());
    valid.then_some(extension)
}

/// Saves a playground video or image wherever the user points the save dialog.
///
/// Returns the destination path, or `None` when the dialog was cancelled.
#[tauri::command]
#[specta::specta]
pub async fn save_playground_media_as(
    app: AppHandle,
    source_path: Option<String>,
    source_url: Option<String>,
    suggested_name: String,
) -> Result<Option<String>, String> {
    let source = MediaSource::parse(source_path, source_url)?;
    let mut file_name = sanitize_file_name(&suggested_name);
    // The dialog only appends an extension when the default name already has
    // one, so borrow it from the source instead of saving an extensionless file.
    let extension = match extension_of(&file_name) {
        Some(extension) => extension,
        None => match source.extension() {
            Some(extension) => {
                file_name = format!("{file_name}.{extension}");
                extension
            }
            None => "bin".to_string(),
        },
    };

    // The dialog blocks until the user answers, so keep it off the async worker.
    let dialog_app = app.clone();
    let dialog_name = file_name.clone();
    let filter_name = extension.to_uppercase();
    let filter_extension = extension.clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        dialog_app
            .dialog()
            .file()
            .set_file_name(dialog_name)
            .add_filter(filter_name, &[filter_extension.as_str()])
            .blocking_save_file()
    })
    .await
    .map_err(|error| format!("Failed to open the save dialog: {error}"))?;

    let Some(picked) = picked else {
        return Ok(None);
    };
    let destination = picked
        .into_path()
        .map_err(|error| format!("Invalid save destination: {error}"))?;

    match source {
        MediaSource::Local(path) => copy_file(path, destination.clone()).await?,
        MediaSource::Remote(url) => download_file(&url, destination.clone()).await?,
    }

    Ok(Some(destination.to_string_lossy().to_string()))
}

async fn copy_file(source: PathBuf, destination: PathBuf) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::copy(&source, &destination)
            .map(|_| ())
            .map_err(|error| format!("Failed to copy the media file: {error}"))
    })
    .await
    .map_err(|error| format!("Failed to copy the media file: {error}"))?
}

async fn download_file(url: &str, destination: PathBuf) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .map_err(|error| format!("Failed to build the HTTP client: {error}"))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("Failed to download the media file: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "The provider returned HTTP {status} for the media file."
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Failed to read the media file: {error}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        std::fs::write(&destination, &bytes)
            .map_err(|error| format!("Failed to write the media file: {error}"))
    })
    .await
    .map_err(|error| format!("Failed to write the media file: {error}"))?
}
