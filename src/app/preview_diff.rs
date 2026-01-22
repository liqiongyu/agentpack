use crate::targets::TargetRoot;

const UNIFIED_DIFF_MAX_BYTES: usize = 100 * 1024;

#[derive(serde::Serialize)]
pub(crate) struct PreviewDiffFile {
    pub(crate) target: String,
    pub(crate) root: String,
    pub(crate) root_posix: String,
    pub(crate) path: String,
    pub(crate) path_posix: String,
    pub(crate) op: crate::deploy::Op,
    pub(crate) before_hash: Option<String>,
    pub(crate) after_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) unified: Option<String>,
}

pub(crate) fn preview_diff_files(
    plan: &crate::deploy::PlanResult,
    desired: &crate::deploy::DesiredState,
    roots: &[TargetRoot],
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<PreviewDiffFile>> {
    let mut out = Vec::new();

    for c in &plan.changes {
        let abs_path = std::path::PathBuf::from(&c.path);
        let root_idx = crate::roots::best_root_idx(roots, &c.target, &abs_path);
        let root_path = root_idx
            .and_then(|idx| roots.get(idx))
            .map(|r| r.root.as_path());
        let root = root_path
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        let root_posix = root_path
            .map(crate::paths::path_to_posix_string)
            .unwrap_or_else(|| "<unknown>".to_string());

        let rel_path = root_idx
            .and_then(|idx| roots.get(idx))
            .and_then(|r| abs_path.strip_prefix(&r.root).ok())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| c.path.clone());
        let rel_path_posix = rel_path.replace('\\', "/");

        let before_hash = c.before_sha256.as_ref().map(|h| format!("sha256:{h}"));
        let after_hash = c.after_sha256.as_ref().map(|h| format!("sha256:{h}"));

        let mut unified: Option<String> = None;
        if matches!(c.op, crate::deploy::Op::Create | crate::deploy::Op::Update) {
            let before_bytes = std::fs::read(&abs_path).unwrap_or_default();
            let tp = crate::deploy::TargetPath {
                target: c.target.clone(),
                path: abs_path.clone(),
            };
            if let Some(df) = desired.get(&tp) {
                match (
                    std::str::from_utf8(&before_bytes).ok(),
                    std::str::from_utf8(&df.bytes).ok(),
                ) {
                    (Some(from), Some(to)) => {
                        let from_name = format!("a/{rel_path}");
                        let to_name = format!("b/{rel_path}");
                        let diff = crate::diff::unified_diff(from, to, &from_name, &to_name);
                        if diff.len() > UNIFIED_DIFF_MAX_BYTES {
                            warnings.push(format!(
                                "preview diff omitted for {} {} (over {} bytes)",
                                c.target, rel_path, UNIFIED_DIFF_MAX_BYTES
                            ));
                        } else {
                            unified = Some(diff);
                        }
                    }
                    _ => {
                        warnings.push(format!(
                            "preview diff omitted for {} {} (binary or non-utf8)",
                            c.target, rel_path
                        ));
                    }
                }
            }
        }

        out.push(PreviewDiffFile {
            target: c.target.clone(),
            root,
            root_posix,
            path: rel_path,
            path_posix: rel_path_posix,
            op: c.op.clone(),
            before_hash,
            after_hash,
            unified,
        });
    }

    Ok(out)
}
