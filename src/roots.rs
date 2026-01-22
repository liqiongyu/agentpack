use std::path::Path;

use crate::targets::TargetRoot;

pub(crate) fn best_root_idx(roots: &[TargetRoot], target: &str, path: &Path) -> Option<usize> {
    roots
        .iter()
        .enumerate()
        .filter(|(_, r)| r.target == target)
        .filter(|(_, r)| path.strip_prefix(&r.root).is_ok())
        .max_by_key(|(_, r)| r.root.components().count())
        .map(|(idx, _)| idx)
}
