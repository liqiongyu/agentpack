use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRoot {
    pub target: String,
    pub root: PathBuf,
    pub scan_extras: bool,
}

pub fn dedup_roots(mut roots: Vec<TargetRoot>) -> Vec<TargetRoot> {
    roots.sort_by(|a, b| {
        (a.target.as_str(), a.root.as_os_str()).cmp(&(b.target.as_str(), b.root.as_os_str()))
    });
    roots.dedup_by(|a, b| a.target == b.target && a.root == b.root);
    roots
}

pub fn best_root_for<'a>(
    roots: &'a [TargetRoot],
    target: &str,
    path: &Path,
) -> Option<&'a TargetRoot> {
    roots
        .iter()
        .filter(|r| r.target == target)
        .filter(|r| path.strip_prefix(&r.root).is_ok())
        .max_by_key(|r| r.root.components().count())
}
