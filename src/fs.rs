use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use tempfile::NamedTempFile;
use walkdir::WalkDir;

use crate::user_error::UserError;

fn env_truthy(var_name: &str) -> bool {
    let Ok(val) = std::env::var(var_name) else {
        return false;
    };

    is_truthy(&val)
}

fn is_truthy(val: &str) -> bool {
    let val = val.trim();
    val == "1"
        || val.eq_ignore_ascii_case("true")
        || val.eq_ignore_ascii_case("yes")
        || val.eq_ignore_ascii_case("y")
        || val.eq_ignore_ascii_case("on")
}

fn fsync_enabled() -> bool {
    env_truthy("AGENTPACK_FSYNC")
}

#[cfg(unix)]
fn sync_dir_best_effort(dir: &Path) -> anyhow::Result<()> {
    use std::fs::File;

    let f = File::open(dir).with_context(|| format!("open dir {}", dir.display()))?;
    match f.sync_all() {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::InvalidInput => {}
        Err(e) => return Err(e).with_context(|| format!("sync dir {}", dir.display())),
    }
    Ok(())
}

pub fn copy_tree(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if src.is_file() {
        let file_name = src
            .file_name()
            .with_context(|| format!("invalid file path: {}", src.display()))?;
        let dst_file = dst.join(file_name);
        copy_file(src, &dst_file)?;
        return Ok(());
    }

    for entry in WalkDir::new(src).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }

        let rel = entry.path().strip_prefix(src).with_context(|| {
            format!(
                "path {} is not under {}",
                entry.path().display(),
                src.display()
            )
        })?;
        if rel
            .components()
            .any(|c| c.as_os_str() == ".git" || c.as_os_str() == ".agentpack")
        {
            continue;
        }
        let dst_path = dst.join(rel);
        copy_file(entry.path(), &dst_path)?;
    }

    Ok(())
}

pub fn copy_tree_missing_only(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if src.is_file() {
        let file_name = src
            .file_name()
            .with_context(|| format!("invalid file path: {}", src.display()))?;
        let dst_file = dst.join(file_name);
        if dst_file.exists() {
            return Ok(());
        }
        copy_file(src, &dst_file)?;
        return Ok(());
    }

    for entry in WalkDir::new(src).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }

        let rel = entry.path().strip_prefix(src).with_context(|| {
            format!(
                "path {} is not under {}",
                entry.path().display(),
                src.display()
            )
        })?;
        if rel
            .components()
            .any(|c| c.as_os_str() == ".git" || c.as_os_str() == ".agentpack")
        {
            continue;
        }
        let dst_path = dst.join(rel);
        if dst_path.exists() {
            continue;
        }
        copy_file(entry.path(), &dst_path)?;
    }

    Ok(())
}

pub fn copy_file(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::copy(src, dst)
        .with_context(|| format!("copy {} -> {}", src.display(), dst.display()))?;
    Ok(())
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    write_atomic_impl(path, bytes, fsync_enabled()).map_err(|err| classify_write_error(path, err))
}

fn write_atomic_impl(path: &Path, bytes: &[u8], fsync: bool) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("invalid path: {}", path.display()))?;
    std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;

    let mut tmp = NamedTempFile::new_in(parent).context("create temp file")?;
    tmp.write_all(bytes).context("write temp file")?;
    tmp.flush().context("flush temp file")?;

    if fsync {
        tmp.as_file()
            .sync_all()
            .context("sync temp file (AGENTPACK_FSYNC=1)")?;
    }

    tmp.persist(path)
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e.error))
        .with_context(|| format!("persist {}", path.display()))?;

    #[cfg(unix)]
    if fsync {
        sync_dir_best_effort(parent).context("sync parent dir (AGENTPACK_FSYNC=1)")?;
    }

    Ok(())
}

fn classify_write_error(path: &Path, err: anyhow::Error) -> anyhow::Error {
    let Some(io) = err.chain().find_map(|e| e.downcast_ref::<std::io::Error>()) else {
        return err;
    };

    let path_str = path.to_string_lossy().to_string();
    let path_posix = path.to_string_lossy().replace('\\', "/");

    let io_kind = io.kind();
    let raw_os_error = io.raw_os_error();

    if io_kind == std::io::ErrorKind::PermissionDenied {
        return anyhow::Error::new(
            UserError::new(
                "E_IO_PERMISSION_DENIED",
                format!("permission denied writing {}", path.display()),
            )
            .with_details(serde_json::json!({
                "path": path_str,
                "path_posix": path_posix,
                "io_kind": format!("{io_kind:?}"),
                "raw_os_error": raw_os_error,
                "hint": "ensure the destination path is writable (and not read-only) and retry",
            })),
        );
    }

    #[cfg(windows)]
    {
        // Windows error codes: https://learn.microsoft.com/en-us/windows/win32/debug/system-error-codes
        // - 5: ERROR_ACCESS_DENIED
        // - 32: ERROR_SHARING_VIOLATION
        // - 123: ERROR_INVALID_NAME
        // - 161: ERROR_BAD_PATHNAME
        // - 206: ERROR_FILENAME_EXCED_RANGE
        // - 111: ERROR_BUFFER_OVERFLOW
        if let Some(code) = raw_os_error {
            match code {
                5 | 32 => {
                    return anyhow::Error::new(
                        UserError::new(
                            "E_IO_PERMISSION_DENIED",
                            format!("permission denied writing {}", path.display()),
                        )
                        .with_details(serde_json::json!({
                            "path": path_str,
                            "path_posix": path_posix,
                            "io_kind": format!("{io_kind:?}"),
                            "raw_os_error": code,
                            "hint": "ensure the destination path is writable and not locked by another process, then retry",
                        })),
                    );
                }
                123 => {
                    return anyhow::Error::new(
                        UserError::new(
                            "E_IO_INVALID_PATH",
                            format!("invalid destination path {}", path.display()),
                        )
                        .with_details(serde_json::json!({
                            "path": path_str,
                            "path_posix": path_posix,
                            "io_kind": format!("{io_kind:?}"),
                            "raw_os_error": code,
                            "hint": "remove invalid characters from the destination path and retry",
                        })),
                    );
                }
                161 => {
                    return anyhow::Error::new(
                        UserError::new(
                            "E_IO_INVALID_PATH",
                            format!("invalid destination path {}", path.display()),
                        )
                        .with_details(serde_json::json!({
                            "path": path_str,
                            "path_posix": path_posix,
                            "io_kind": format!("{io_kind:?}"),
                            "raw_os_error": code,
                            "hint": "remove invalid characters from the destination path and retry",
                        })),
                    );
                }
                111 | 206 => {
                    return anyhow::Error::new(
                        UserError::new(
                            "E_IO_PATH_TOO_LONG",
                            format!("destination path is too long {}", path.display()),
                        )
                        .with_details(serde_json::json!({
                            "path": path_str,
                            "path_posix": path_posix,
                            "io_kind": format!("{io_kind:?}"),
                            "raw_os_error": code,
                            "hint": "use a shorter workspace/home path (or enable long paths on Windows) and retry",
                        })),
                    );
                }
                _ => {}
            }
        }

        // Heuristics for cases where Rust surfaces a non-specific IO error (or no raw_os_error),
        // but the path is clearly invalid or exceeds common Windows path limits.
        let looks_invalid = path_str.contains('<')
            || path_str.contains('>')
            || path_str.contains('|')
            || path_str.contains('"')
            || path_str.contains('?')
            || path_str.contains('*');
        let looks_too_long = path_str.len() >= 260;

        if looks_too_long {
            return anyhow::Error::new(
                UserError::new(
                    "E_IO_PATH_TOO_LONG",
                    format!("destination path is too long {}", path.display()),
                )
                .with_details(serde_json::json!({
                    "path": path_str,
                    "path_posix": path_posix,
                    "io_kind": format!("{io_kind:?}"),
                    "raw_os_error": raw_os_error,
                    "hint": "use a shorter workspace/home path (or enable long paths on Windows) and retry",
                })),
            );
        }

        if looks_invalid || io_kind == std::io::ErrorKind::InvalidInput {
            return anyhow::Error::new(
                UserError::new(
                    "E_IO_INVALID_PATH",
                    format!("invalid destination path {}", path.display()),
                )
                .with_details(serde_json::json!({
                    "path": path_str,
                    "path_posix": path_posix,
                    "io_kind": format!("{io_kind:?}"),
                    "raw_os_error": raw_os_error,
                    "hint": "remove invalid characters from the destination path and retry",
                })),
            );
        }
    }

    err
}

pub fn list_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if entry
            .path()
            .components()
            .any(|c| c.as_os_str() == ".agentpack" || c.as_os_str() == ".git")
        {
            continue;
        }
        if entry.file_type().is_file() {
            out.push(entry.into_path());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_truthy_accepts_common_true_values() {
        assert!(is_truthy("1"));
        assert!(is_truthy("true"));
        assert!(is_truthy("TRUE"));
        assert!(is_truthy(" yes "));
        assert!(is_truthy("On"));
        assert!(!is_truthy("0"));
        assert!(!is_truthy("false"));
        assert!(!is_truthy(""));
    }

    #[test]
    fn write_atomic_with_fsync_enabled_does_not_fail() {
        let td = tempfile::tempdir().unwrap();
        let path = td.path().join("out.txt");
        write_atomic_impl(&path, b"hello", true).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"hello");
    }
}
