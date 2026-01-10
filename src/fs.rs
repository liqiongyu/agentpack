use std::path::{Path, PathBuf};

use anyhow::Context as _;
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
            if entry.file_name() == ".git" {
                continue;
            }
            continue;
        }
        if entry.path().components().any(|c| c.as_os_str() == ".git") {
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

pub fn list_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_file() {
            out.push(entry.into_path());
        }
    }
    Ok(out)
}
