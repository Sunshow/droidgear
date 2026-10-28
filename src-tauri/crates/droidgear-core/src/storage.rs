use std::io::Write;
use std::path::{Path, PathBuf};

/// Replace `path` atomically.
///
/// Writes a sibling temporary file with a unique suffix, then renames it over
/// the target. The unique suffix means two concurrent writers can never share
/// a temp file; the rename itself is atomic on POSIX filesystems, so readers
/// see either the old or the new complete file.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    atomic_write_with_mode(path, bytes, None)
}

/// Atomically replace `path` with an owner-only (0600) document on Unix.
///
/// Used for files the DeepSeek Harness also owns and writes with mode 0600
/// (`cordis.patch.yml`, `.credentials.yaml`).
pub fn atomic_write_private(path: &Path, bytes: &[u8]) -> Result<(), String> {
    #[cfg(unix)]
    {
        atomic_write_with_mode(path, bytes, Some(0o600))
    }
    #[cfg(not(unix))]
    {
        atomic_write_with_mode(path, bytes, None)
    }
}

// On non-Unix targets the `mode` argument is compiled out of use below.
#[cfg_attr(not(unix), allow(unused_variables))]
fn atomic_write_with_mode(path: &Path, bytes: &[u8], mode: Option<u32>) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    let temp_path = unique_temp_path(parent, path);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    let result = options
        .open(&temp_path)
        .and_then(|mut file| file.write_all(bytes))
        .and_then(|()| std::fs::rename(&temp_path, path));
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            Err(format!("Failed to write file: {e}"))
        }
    }
}

fn unique_temp_path(parent: &Path, target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let pid = std::process::id();
    let mut counter: u64 = 0;
    loop {
        let suffix = if counter == 0 {
            format!("{pid}-{nanos}")
        } else {
            format!("{pid}-{nanos}-{counter}")
        };
        let candidate = parent.join(format!(".{name}.{suffix}.tmp"));
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_content() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = dir.path().join("config.yml");
        atomic_write(&path, b"first").expect("first write");
        atomic_write(&path, b"second").expect("second write");
        assert_eq!(std::fs::read(&path).expect("read"), b"second");
    }

    #[test]
    fn atomic_write_leaves_no_temp_file() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = dir.path().join("config.yml");
        atomic_write(&path, b"content").expect("write");
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "leftover temp files: {leftovers:?}");
    }

    #[cfg(unix)]
    #[test]
    fn private_write_uses_owner_only_mode() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = dir.path().join("secret.yml");
        atomic_write_private(&path, b"secret").expect("write");
        let mode = std::fs::metadata(&path).expect("meta").permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
