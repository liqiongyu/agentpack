use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use tempfile::NamedTempFile;
use walkdir::WalkDir;

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
            if entry.file_name() == ".git" || entry.file_name() == ".agentpack" {
                continue;
            }
            continue;
        }
        if entry
            .path()
            .components()
            .any(|c| c.as_os_str() == ".git" || c.as_os_str() == ".agentpack")
        {
            continue;
        }

        let rel = entry.path().strip_prefix(src).unwrap_or(entry.path());
        let dst_path = dst.join(rel);
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
    let parent = path
        .parent()
        .with_context(|| format!("invalid path: {}", path.display()))?;
    std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;

    let mut tmp = NamedTempFile::new_in(parent).context("create temp file")?;
    tmp.write_all(bytes).context("write temp file")?;
    tmp.flush().context("flush temp file")?;

    tmp.persist(path)
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e.error))
        .with_context(|| format!("persist {}", path.display()))?;

    Ok(())
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
