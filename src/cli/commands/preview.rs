use crate::deploy::TargetPath;
use crate::deploy::load_managed_paths_from_snapshot;
use crate::deploy::plan as compute_plan;
use crate::diff::unified_diff;
use crate::engine::Engine;
use crate::output::{JsonEnvelope, print_json};
use crate::state::latest_snapshot;

use super::Ctx;

const UNIFIED_DIFF_MAX_BYTES: usize = 100 * 1024;

#[derive(serde::Serialize)]
struct PreviewDiffFile {
    target: String,
    root: String,
    root_posix: String,
    path: String,
    path_posix: String,
    op: crate::deploy::Op,
    before_hash: Option<String>,
    after_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unified: Option<String>,
}

pub(crate) fn run(ctx: &Ctx<'_>, diff: bool) -> anyhow::Result<()> {
    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let targets = super::super::util::selected_targets(&engine.manifest, &ctx.cli.target)?;
    let render = engine.desired_state(&ctx.cli.profile, &ctx.cli.target)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;
    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
    warnings.extend(managed_paths_from_manifest.warnings);
    let managed_paths_from_manifest = managed_paths_from_manifest.managed_paths;
    let managed_paths = if !managed_paths_from_manifest.is_empty() {
        Some(super::super::util::filter_managed(
            managed_paths_from_manifest,
            &ctx.cli.target,
        ))
    } else {
        latest_snapshot(&engine.home, &["deploy", "rollback"])?
            .as_ref()
            .map(load_managed_paths_from_snapshot)
            .transpose()?
            .map(|m| super::super::util::filter_managed(m, &ctx.cli.target))
    };

    let plan = compute_plan(&desired, managed_paths.as_ref())?;

    if ctx.cli.json {
        let plan_changes = plan.changes.clone();
        let plan_summary = plan.summary.clone();
        let mut data = serde_json::json!({
            "profile": ctx.cli.profile,
            "targets": targets,
            "plan": {
                "changes": plan_changes,
                "summary": plan_summary,
            },
        });
        if diff {
            let files = preview_diff_files(&plan, &desired, &roots, &mut warnings)?;
            data["diff"] = serde_json::json!({
                "changes": plan.changes,
                "summary": plan.summary,
                "files": files,
            });
        }

        let mut envelope = JsonEnvelope::ok("preview", data);
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else {
        for w in warnings {
            eprintln!("Warning: {w}");
        }
        println!(
            "Plan: +{} ~{} -{}",
            plan.summary.create, plan.summary.update, plan.summary.delete
        );
        if diff {
            super::super::util::print_diff(&plan, &desired)?;
        } else {
            for c in &plan.changes {
                println!("{:?} {} {}", c.op, c.target, c.path);
            }
        }
    }

    Ok(())
}

fn preview_diff_files(
    plan: &crate::deploy::PlanResult,
    desired: &crate::deploy::DesiredState,
    roots: &[crate::targets::TargetRoot],
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<PreviewDiffFile>> {
    let mut out = Vec::new();

    for c in &plan.changes {
        let abs_path = std::path::PathBuf::from(&c.path);
        let root_idx = super::super::util::best_root_idx(roots, &c.target, &abs_path);
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
            let tp = TargetPath {
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
                        let diff = unified_diff(from, to, &from_name, &to_name);
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
