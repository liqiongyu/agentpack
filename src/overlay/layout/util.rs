use std::path::{Path, PathBuf};

use anyhow::Context as _;

pub(crate) fn validate_posix_relpath(relpath: &str) -> bool {
    if relpath.is_empty() || relpath.starts_with('/') {
        return false;
    }
    relpath
        .split('/')
        .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

pub(crate) fn join_posix(root: &Path, rel_posix: &str) -> PathBuf {
    let mut out = root.to_path_buf();
    for part in rel_posix.split('/') {
        out.push(part);
    }
    out
}

pub(crate) fn path_relative_posix(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn delete_overlay_file(
    overlay_dir: &Path,
    file: &Path,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        return Ok(());
    }

    std::fs::remove_file(file).with_context(|| format!("remove {}", file.display()))?;
    prune_empty_parents(
        file.parent().with_context(|| "missing file parent")?,
        overlay_dir,
    )?;
    Ok(())
}

fn prune_empty_parents(mut dir: &Path, stop: &Path) -> anyhow::Result<()> {
    while dir != stop {
        let mut entries =
            std::fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))?;
        if entries.next().is_some() {
            break;
        }
        std::fs::remove_dir(dir).with_context(|| format!("remove_dir {}", dir.display()))?;
        dir = dir.parent().with_context(|| "missing parent")?;
    }
    Ok(())
}
