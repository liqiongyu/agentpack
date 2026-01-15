use std::fmt::Write as _;

use anyhow::Context as _;

use crate::deploy::load_managed_paths_from_snapshot;
use crate::deploy::plan as compute_plan;
use crate::deploy::{DesiredState, ManagedPaths, TargetPath};
use crate::engine::Engine;
use crate::hash::sha256_hex;
use crate::state::latest_snapshot;
use crate::targets::TargetRoot;
use crate::user_error::UserError;

#[derive(Debug)]
pub struct ReadOnlyTextViews {
    pub warnings: Vec<String>,
    pub plan: String,
    pub diff: String,
    pub status: String,
}

pub fn collect_read_only_text_views(
    engine: &Engine,
    profile: &str,
    target_filter: &str,
) -> anyhow::Result<ReadOnlyTextViews> {
    let _targets = selected_targets(&engine.manifest, target_filter)?;

    let render = engine.desired_state(profile, target_filter)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;

    let managed_paths = managed_paths_for_plan(engine, &roots, target_filter, &mut warnings)?;
    let plan = compute_plan(&desired, managed_paths.as_ref())?;

    let plan_text = render_plan_text(&warnings, &plan)?;
    let diff_text = render_diff_text(&warnings, &plan, &desired)?;
    let status_text = render_status_text(&warnings, &desired, &roots)?;

    Ok(ReadOnlyTextViews {
        warnings,
        plan: plan_text,
        diff: diff_text,
        status: status_text,
    })
}

fn selected_targets(
    manifest: &crate::config::Manifest,
    target_filter: &str,
) -> anyhow::Result<Vec<String>> {
    let mut known: Vec<String> = manifest.targets.keys().cloned().collect();
    known.sort();

    match target_filter {
        "all" => Ok(known),
        "codex" | "claude_code" | "cursor" | "vscode" => {
            if !manifest.targets.contains_key(target_filter) {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_CONFIG_INVALID",
                        format!("target not configured: {target_filter}"),
                    )
                    .with_details(serde_json::json!( {
                        "target": target_filter,
                        "hint": "add the target under `targets:` in agentpack.yaml",
                    })),
                ));
            }
            Ok(vec![target_filter.to_string()])
        }
        other => Err(anyhow::Error::new(
            UserError::new(
                "E_TARGET_UNSUPPORTED",
                format!("unsupported --target: {other}"),
            )
            .with_details(serde_json::json!( {
                "target": other,
                "allowed": ["all","codex","claude_code","cursor","vscode"],
            })),
        )),
    }
}

fn managed_paths_for_plan(
    engine: &Engine,
    roots: &[TargetRoot],
    target_filter: &str,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Option<ManagedPaths>> {
    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(roots)?;
    warnings.extend(managed_paths_from_manifest.warnings);
    let managed_paths_from_manifest = managed_paths_from_manifest.managed_paths;

    if !managed_paths_from_manifest.is_empty() {
        return Ok(Some(filter_managed(
            managed_paths_from_manifest,
            target_filter,
        )));
    }

    let latest = latest_snapshot(&engine.home, &["deploy", "rollback"])?;
    Ok(latest
        .as_ref()
        .map(load_managed_paths_from_snapshot)
        .transpose()?
        .map(|m| filter_managed(m, target_filter)))
}

fn filter_managed(managed: ManagedPaths, target_filter: &str) -> ManagedPaths {
    managed
        .into_iter()
        .filter(|tp| target_filter == "all" || tp.target == target_filter)
        .collect()
}

fn render_warnings(out: &mut String, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }

    for w in warnings {
        let _ = writeln!(out, "Warning: {w}");
    }
    let _ = writeln!(out);
}

fn render_plan_text(
    warnings: &[String],
    plan: &crate::deploy::PlanResult,
) -> anyhow::Result<String> {
    let mut out = String::new();
    render_warnings(&mut out, warnings);

    writeln!(
        out,
        "Plan: +{} ~{} -{}",
        plan.summary.create, plan.summary.update, plan.summary.delete
    )
    .context("write plan summary")?;
    for c in &plan.changes {
        writeln!(out, "{:?} {} {}", c.op, c.target, c.path).context("write plan change")?;
    }

    Ok(out)
}

fn render_diff_text(
    warnings: &[String],
    plan: &crate::deploy::PlanResult,
    desired: &DesiredState,
) -> anyhow::Result<String> {
    let mut out = String::new();
    render_warnings(&mut out, warnings);

    if plan.changes.is_empty() {
        writeln!(out, "(no changes)").context("write diff no-op")?;
        return Ok(out);
    }

    for c in &plan.changes {
        let path = std::path::PathBuf::from(&c.path);
        let desired_key = TargetPath {
            target: c.target.clone(),
            path: path.clone(),
        };

        let before_text = if matches!(c.op, crate::deploy::Op::Create) {
            Some(String::new())
        } else {
            crate::deploy::read_text(&path)?
        };
        let after_text = if matches!(c.op, crate::deploy::Op::Delete) {
            Some(String::new())
        } else {
            desired
                .get(&desired_key)
                .and_then(|f| String::from_utf8(f.bytes.clone()).ok())
        };

        writeln!(out).context("write diff spacer")?;
        writeln!(out, "=== {} {} ===", c.target, c.path).context("write diff header")?;
        match (before_text, after_text) {
            (Some(from), Some(to)) => {
                out.push_str(&crate::diff::unified_diff(
                    &from,
                    &to,
                    &format!("before: {}", c.path),
                    &format!("after: {}", c.path),
                ));
            }
            _ => {
                writeln!(out, "(binary or non-utf8 content; diff omitted)")
                    .context("write diff omitted")?;
            }
        }
    }

    Ok(out)
}

#[derive(Default, Clone, Copy)]
struct DriftSummary {
    modified: u64,
    missing: u64,
    extra: u64,
}

#[derive(Debug)]
struct DriftItem {
    target: String,
    root: Option<String>,
    path: String,
    kind: String,
}

fn best_root_idx(roots: &[TargetRoot], target: &str, path: &std::path::Path) -> Option<usize> {
    roots
        .iter()
        .enumerate()
        .filter(|(_, r)| r.target == target)
        .filter(|(_, r)| path.strip_prefix(&r.root).is_ok())
        .max_by_key(|(_, r)| r.root.components().count())
        .map(|(idx, _)| idx)
}

fn render_status_text(
    warnings: &[String],
    desired: &DesiredState,
    roots: &[TargetRoot],
) -> anyhow::Result<String> {
    let mut warnings: Vec<String> = warnings.to_vec();

    let mut manifests: Vec<Option<crate::target_manifest::TargetManifest>> =
        vec![None; roots.len()];
    for (idx, root) in roots.iter().enumerate() {
        let path = crate::target_manifest::manifest_path(&root.root);
        if !path.exists() {
            continue;
        }

        let (manifest, manifest_warnings) =
            crate::target_manifest::read_target_manifest_soft(&path, &root.target);
        warnings.extend(manifest_warnings);
        manifests[idx] = manifest;
    }

    let any_manifest = manifests.iter().any(Option::is_some);
    if !any_manifest {
        warnings.push(
            "no target manifests found; drift may be inaccurate (run deploy --apply to write manifests)"
                .to_string(),
        );
    }

    let mut drift: Vec<DriftItem> = Vec::new();
    let mut summary = DriftSummary::default();

    if !any_manifest {
        for (tp, desired_file) in desired {
            let expected = format!("sha256:{}", sha256_hex(&desired_file.bytes));
            let root = best_root_idx(roots, &tp.target, &tp.path)
                .and_then(|idx| roots.get(idx))
                .map(|r| r.root.as_path());
            match std::fs::read(&tp.path) {
                Ok(actual_bytes) => {
                    let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                    if actual != expected {
                        summary.modified += 1;
                        drift.push(DriftItem {
                            target: tp.target.clone(),
                            root: root.map(|p| p.to_string_lossy().to_string()),
                            path: tp.path.to_string_lossy().to_string(),
                            kind: "modified".to_string(),
                        });
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    summary.missing += 1;
                    drift.push(DriftItem {
                        target: tp.target.clone(),
                        root: root.map(|p| p.to_string_lossy().to_string()),
                        path: tp.path.to_string_lossy().to_string(),
                        kind: "missing".to_string(),
                    });
                }
                Err(err) => return Err(err).context("read deployed file"),
            }
        }
    } else {
        let mut desired_by_root: Vec<Vec<(&TargetPath, &crate::deploy::DesiredFile)>> =
            vec![Vec::new(); roots.len()];
        for (tp, desired_file) in desired {
            let Some(root_idx) = best_root_idx(roots, &tp.target, &tp.path) else {
                continue;
            };
            desired_by_root[root_idx].push((tp, desired_file));
        }

        for (idx, root) in roots.iter().enumerate() {
            let Some(manifest) = &manifests[idx] else {
                if desired_by_root[idx].is_empty() {
                    continue;
                }
                warnings.push(format!(
                    "no usable target manifest for {} {}; drift may be incomplete (run deploy --apply to write manifests)",
                    root.target,
                    root.root.display()
                ));

                for (tp, desired_file) in &desired_by_root[idx] {
                    let expected = format!("sha256:{}", sha256_hex(&desired_file.bytes));
                    match std::fs::read(&tp.path) {
                        Ok(actual_bytes) => {
                            let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                            if actual != expected {
                                summary.modified += 1;
                                drift.push(DriftItem {
                                    target: tp.target.clone(),
                                    root: Some(root.root.to_string_lossy().to_string()),
                                    path: tp.path.to_string_lossy().to_string(),
                                    kind: "modified".to_string(),
                                });
                            }
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                            summary.missing += 1;
                            drift.push(DriftItem {
                                target: tp.target.clone(),
                                root: Some(root.root.to_string_lossy().to_string()),
                                path: tp.path.to_string_lossy().to_string(),
                                kind: "missing".to_string(),
                            });
                        }
                        Err(err) => return Err(err).context("read deployed file"),
                    }
                }
                continue;
            };

            let mut managed_paths = crate::deploy::ManagedPaths::new();
            for f in &manifest.managed_files {
                let rel_path = std::path::Path::new(&f.path);
                if rel_path.is_absolute()
                    || rel_path
                        .components()
                        .any(|c| matches!(c, std::path::Component::ParentDir))
                {
                    warnings.push(format!(
                        "target manifest ({}): skipped invalid entry path {:?} in {}",
                        root.target,
                        f.path,
                        crate::target_manifest::manifest_path(&root.root).display(),
                    ));
                    continue;
                }

                managed_paths.insert(TargetPath {
                    target: root.target.clone(),
                    path: root.root.join(&f.path),
                });
            }

            for tp in &managed_paths {
                let expected = desired
                    .get(tp)
                    .map(|f| format!("sha256:{}", sha256_hex(&f.bytes)));
                match std::fs::read(&tp.path) {
                    Ok(actual_bytes) => {
                        let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                        if let Some(exp) = &expected {
                            if &actual != exp {
                                summary.modified += 1;
                                drift.push(DriftItem {
                                    target: tp.target.clone(),
                                    root: Some(root.root.to_string_lossy().to_string()),
                                    path: tp.path.to_string_lossy().to_string(),
                                    kind: "modified".to_string(),
                                });
                            }
                        } else {
                            summary.extra += 1;
                            drift.push(DriftItem {
                                target: tp.target.clone(),
                                root: Some(root.root.to_string_lossy().to_string()),
                                path: tp.path.to_string_lossy().to_string(),
                                kind: "extra".to_string(),
                            });
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                        if expected.is_some() {
                            summary.missing += 1;
                            drift.push(DriftItem {
                                target: tp.target.clone(),
                                root: Some(root.root.to_string_lossy().to_string()),
                                path: tp.path.to_string_lossy().to_string(),
                                kind: "missing".to_string(),
                            });
                        }
                    }
                    Err(err) => return Err(err).context("read deployed file"),
                }
            }

            if !root.scan_extras {
                continue;
            }
            if !root.root.exists() {
                continue;
            }

            let mut files = crate::fs::list_files(&root.root)?;
            files.sort();
            for path in files {
                if path.file_name().and_then(|s| s.to_str())
                    == Some(crate::target_manifest::TARGET_MANIFEST_FILENAME)
                {
                    continue;
                }

                let tp = TargetPath {
                    target: root.target.clone(),
                    path: path.clone(),
                };
                if managed_paths.contains(&tp) {
                    continue;
                }

                summary.extra += 1;
                drift.push(DriftItem {
                    target: tp.target.clone(),
                    root: Some(root.root.to_string_lossy().to_string()),
                    path: tp.path.to_string_lossy().to_string(),
                    kind: "extra".to_string(),
                });
            }
        }
    }

    drift.sort_by(|a, b| {
        (
            a.target.as_str(),
            a.root.as_deref().unwrap_or(""),
            a.path.as_str(),
        )
            .cmp(&(
                b.target.as_str(),
                b.root.as_deref().unwrap_or(""),
                b.path.as_str(),
            ))
    });

    let mut out = String::new();
    render_warnings(&mut out, &warnings);
    if drift.is_empty() {
        writeln!(out, "No drift").context("write no drift")?;
        return Ok(out);
    }

    writeln!(out, "Drift ({}):", drift.len()).context("write drift header")?;
    writeln!(
        out,
        "Summary: modified={} missing={} extra={}",
        summary.modified, summary.missing, summary.extra
    )
    .context("write drift summary")?;

    let mut last_group: Option<(String, String)> = None;
    for d in drift {
        let root = d.root.as_deref().unwrap_or("<unknown>");
        let group = (d.target.clone(), root.to_string());
        if last_group.as_ref() != Some(&group) {
            writeln!(out, "Root: {} ({})", root, d.target).context("write drift root")?;
            last_group = Some(group);
        }
        writeln!(out, "- {} {}", d.kind, d.path).context("write drift item")?;
    }

    Ok(out)
}
