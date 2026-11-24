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
    async fn parse(
        source_path: Option<String>,
        source_url: Option<String>,
    ) -> Result<Self, String> {
        if let Some(path) = normalise(source_path) {
            return validate_local_path(&path).map(Self::Local);
        }
        if let Some(url) = normalise(source_url) {
            let url = validate_remote_url(&url)?;
            ensure_remote_target_is_routable(&url).await?;
            return Ok(Self::Remote(url));
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
        "http" | "https" => {
            let host = url.host_str().unwrap_or_default();
            if is_blocked_host(host) {
                return Err("The media URL points at a local address.".to_string());
            }
            if let Some(ip) = parse_literal_ip(host) {
                reject_local_ip(ip)?;
            }
            Ok(url.to_string())
        }
        scheme => Err(format!("Unsupported media URL scheme: {scheme}")),
    }
}

/// The provider decides `source_url`, so a hostile response must not be able to
/// point the downloader at loopback services, cloud metadata endpoints
/// (`169.254.169.254`, `fe80::`) or the `localhost` aliases. Private LAN
/// addresses stay allowed: self-hosted gateways legitimately hand out media
/// URLs on `192.168.x` / `10.x`.
async fn ensure_remote_target_is_routable(url: &str) -> Result<(), String> {
    use std::net::ToSocketAddrs;

    let parsed = reqwest::Url::parse(url).map_err(|error| format!("Invalid media URL: {error}"))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| "The media URL has no host.".to_string())?
        .to_string();

    if let Some(ip) = parse_literal_ip(&host) {
        return reject_local_ip(ip);
    }
    let port = parsed.port_or_known_default().unwrap_or(443);
    let resolved = tauri::async_runtime::spawn_blocking(move || {
        (host.as_str(), port)
            .to_socket_addrs()
            .map(|addrs| addrs.map(|addr| addr.ip()).collect::<Vec<_>>())
    })
    .await
    .map_err(|error| format!("Failed to resolve the media host: {error}"))?
    .map_err(|error| format!("Failed to resolve the media host: {error}"))?;

    if resolved.is_empty() {
        return Err("The media host did not resolve to any address.".to_string());
    }
    for ip in resolved {
        reject_local_ip(ip)?;
    }
    Ok(())
}

fn is_blocked_host(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    host == "localhost"
        || host.ends_with(".localhost")
        || host == "ip6-localhost"
        || host == "metadata.google.internal"
}

/// `Url::host_str` keeps the brackets around IPv6 literals (`[::1]`).
fn parse_literal_ip(host: &str) -> Option<std::net::IpAddr> {
    host.trim_start_matches('[')
        .trim_end_matches(']')
        .parse()
        .ok()
}

fn reject_local_ip(ip: std::net::IpAddr) -> Result<(), String> {
    let blocked = match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_loopback() || v4.is_link_local() || v4.is_unspecified() || v4.is_broadcast()
        }
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                // fe80::/10 link-local range (stable since Rust 1.72).
                || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    };
    if blocked {
        return Err("The media URL points at a local address.".to_string());
    }
    Ok(())
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
    let source = MediaSource::parse(source_path, source_url).await?;
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
        // A redirect must not undo the loopback/metadata check above.
        .redirect(reqwest::redirect::Policy::limited(2))
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
    let final_url = response.url().clone();
    if is_blocked_host(final_url.host_str().unwrap_or_default()) {
        return Err("The media URL points at a local address.".to_string());
    }
    if let Some(ip) = final_url.host_str().and_then(parse_literal_ip) {
        reject_local_ip(ip)?;
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

#[cfg(test)]
mod tests {
    use super::{is_blocked_host, reject_local_ip, validate_remote_url};

    #[test]
    fn remote_media_url_accepts_public_https() {
        assert!(validate_remote_url("https://cdn.example.com/video.mp4?id=1").is_ok());
        // Self-hosted gateways on a LAN stay supported.
        assert!(validate_remote_url("http://192.168.1.10:3000/file.png").is_ok());
        assert!(validate_remote_url("https://10.0.0.5/media/a.png").is_ok());
    }

    #[test]
    fn remote_media_url_rejects_local_targets_and_odd_schemes() {
        for blocked in [
            "http://127.0.0.1:8080/admin",
            "http://localhost:1420/",
            "http://metadata.google.internal/computeMetadata/v1/",
            "http://169.254.169.254/latest/meta-data/",
            "https://[::1]/",
            "https://[fe80::1]/",
            "file:///etc/passwd",
            "data:image/png;base64,AAAA",
        ] {
            assert!(
                validate_remote_url(blocked).is_err(),
                "{blocked} should be rejected"
            );
        }
    }

    #[test]
    fn blocked_hosts_cover_loopback_aliases() {
        assert!(is_blocked_host("localhost"));
        assert!(is_blocked_host("foo.localhost."));
        assert!(!is_blocked_host("example.com"));
    }

    #[test]
    fn link_local_and_loopback_ips_are_rejected() {
        assert!(reject_local_ip("127.0.0.1".parse().unwrap()).is_err());
        assert!(reject_local_ip("169.254.169.254".parse().unwrap()).is_err());
        assert!(reject_local_ip("fe80::1".parse().unwrap()).is_err());
        assert!(reject_local_ip("::1".parse().unwrap()).is_err());
        assert!(reject_local_ip("93.184.216.34".parse().unwrap()).is_ok());
    }
}
