use anyhow::Context as _;

use crate::deploy::TargetPath;
use crate::engine::Engine;
use crate::fs::write_atomic;
use crate::output::{JsonEnvelope, print_json};
use crate::user_error::UserError;

use super::super::args::{EvolveCommands, EvolveScope, OverlayScope};
use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &EvolveCommands) -> anyhow::Result<()> {
    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    match command {
        EvolveCommands::Propose {
            module_id,
            scope,
            branch,
        } => evolve_propose(
            ctx.cli,
            &engine,
            module_id.as_deref(),
            *scope,
            branch.as_deref(),
        ),
    }
}

fn evolve_propose(
    cli: &super::super::args::Cli,
    engine: &Engine,
    module_filter: Option<&str>,
    scope: EvolveScope,
    branch_override: Option<&str>,
) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct ProposalItem {
        module_id: String,
        target: String,
        path: String,
        path_posix: String,
    }

    #[derive(serde::Serialize)]
    struct SkippedItem {
        target: String,
        path: String,
        path_posix: String,
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        module_id: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        module_ids: Vec<String>,
    }

    #[derive(Default, serde::Serialize)]
    struct ProposalSummary {
        drifted_proposeable: u64,
        drifted_skipped: u64,
        skipped_missing: u64,
        skipped_multi_module: u64,
        skipped_read_error: u64,
    }

    let render = engine.desired_state(&cli.profile, &cli.target)?;
    let desired = render.desired;
    let roots = render.roots;

    let mut summary = ProposalSummary::default();
    let mut candidates: Vec<(String, TargetPath, Vec<u8>)> = Vec::new();
    let mut skipped: Vec<SkippedItem> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for (tp, desired_file) in &desired {
        if let Some(filter) = module_filter {
            if !desired_file.module_ids.iter().any(|id| id == filter) {
                continue;
            }
        }

        let actual = match std::fs::read(&tp.path) {
            Ok(bytes) => Some(bytes),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
            Err(err) => {
                summary.skipped_read_error += 1;
                warnings.push(format!(
                    "evolve.propose: skipped {} {}: failed to read deployed file: {err}",
                    tp.target,
                    tp.path.display()
                ));
                continue;
            }
        };

        let is_drifted = match &actual {
            Some(bytes) => bytes != &desired_file.bytes,
            None => true,
        };
        if !is_drifted {
            continue;
        }

        if desired_file.module_ids.len() != 1 {
            summary.drifted_skipped += 1;
            summary.skipped_multi_module += 1;
            skipped.push(SkippedItem {
                target: tp.target.clone(),
                path: tp.path.to_string_lossy().to_string(),
                path_posix: crate::paths::path_to_posix_string(&tp.path),
                reason: "multi_module_output".to_string(),
                module_id: None,
                module_ids: desired_file.module_ids.clone(),
            });
            continue;
        }

        let module_id = desired_file.module_ids[0].clone();
        match actual {
            Some(actual) => {
                summary.drifted_proposeable += 1;
                candidates.push((module_id, tp.clone(), actual));
            }
            None => {
                summary.drifted_skipped += 1;
                summary.skipped_missing += 1;
                skipped.push(SkippedItem {
                    target: tp.target.clone(),
                    path: tp.path.to_string_lossy().to_string(),
                    path_posix: crate::paths::path_to_posix_string(&tp.path),
                    reason: "missing".to_string(),
                    module_id: Some(module_id),
                    module_ids: Vec::new(),
                });
            }
        }
    }

    let mut items: Vec<ProposalItem> = candidates
        .iter()
        .map(|(module_id, tp, _)| ProposalItem {
            module_id: module_id.clone(),
            target: tp.target.clone(),
            path: tp.path.to_string_lossy().to_string(),
            path_posix: crate::paths::path_to_posix_string(&tp.path),
        })
        .collect();
    items.sort_by(|a, b| {
        (a.module_id.as_str(), a.path.as_str()).cmp(&(b.module_id.as_str(), b.path.as_str()))
    });

    skipped.sort_by(|a, b| {
        (a.reason.as_str(), a.target.as_str(), a.path.as_str()).cmp(&(
            b.reason.as_str(),
            b.target.as_str(),
            b.path.as_str(),
        ))
    });

    if items.is_empty() {
        let reason = if skipped.is_empty() {
            "no_drift"
        } else {
            "no_proposeable_drift"
        };

        if cli.json {
            let envelope = JsonEnvelope::ok(
                "evolve.propose",
                serde_json::json!({
                    "created": false,
                    "reason": reason,
                    "summary": summary,
                    "skipped": skipped,
                }),
            );
            let mut envelope = envelope;
            envelope.warnings = warnings;
            print_json(&envelope)?;
        } else {
            for w in warnings {
                eprintln!("Warning: {w}");
            }
            if reason == "no_drift" {
                println!("No drifted managed files to propose");
            } else {
                println!("No proposeable drifted files to propose");
                if !skipped.is_empty() {
                    println!("Skipped drift (not proposeable):");
                    for s in skipped {
                        let who = s
                            .module_id
                            .as_deref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| {
                                if s.module_ids.is_empty() {
                                    "-".to_string()
                                } else {
                                    s.module_ids.join(",")
                                }
                            });
                        println!("- {} {} {} modules={who}", s.reason, s.target, s.path);
                    }
                }
            }
        }
        return Ok(());
    }

    if cli.dry_run {
        if cli.json {
            let envelope = JsonEnvelope::ok(
                "evolve.propose",
                serde_json::json!({
                    "created": false,
                    "reason": "dry_run",
                    "candidates": items,
                    "skipped": skipped,
                    "summary": summary,
                }),
            );
            let mut envelope = envelope;
            envelope.warnings = warnings;
            print_json(&envelope)?;
        } else {
            for w in warnings {
                eprintln!("Warning: {w}");
            }
            println!("Candidates (dry-run):");
            for i in items {
                println!("- {} {} {}", i.module_id, i.target, i.path);
            }
            if !skipped.is_empty() {
                println!("Skipped drift (not proposeable):");
                for s in skipped {
                    let who = s
                        .module_id
                        .as_deref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| {
                            if s.module_ids.is_empty() {
                                "-".to_string()
                            } else {
                                s.module_ids.join(",")
                            }
                        });
                    println!("- {} {} {} modules={who}", s.reason, s.target, s.path);
                }
            }
        }
        return Ok(());
    }

    if cli.json && !cli.yes {
        return Err(UserError::confirm_required("evolve propose"));
    }
    if !cli.json && !cli.yes && !super::super::util::confirm("Create evolve proposal branch?")? {
        println!("Aborted");
        return Ok(());
    }

    let repo_dir = engine.repo.repo_dir.as_path();
    if !repo_dir.join(".git").exists() {
        anyhow::bail!(
            "config repo is not a git repository: {}",
            repo_dir.display()
        );
    }

    let status = crate::git::git_in(repo_dir, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        anyhow::bail!("refusing to propose with a dirty working tree (commit or stash first)");
    }

    let original = crate::git::git_in(repo_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let original = original.trim().to_string();

    let branch = branch_override.map(|s| s.to_string()).unwrap_or_else(|| {
        let nanos = time::OffsetDateTime::now_utc().unix_timestamp_nanos();
        format!("evolve/propose-{nanos}")
    });

    crate::git::git_in(repo_dir, &["checkout", "-b", branch.as_str()])?;

    let mut touched = Vec::new();
    for (module_id, output, actual) in &candidates {
        let Some(module) = engine.manifest.modules.iter().find(|m| m.id == *module_id) else {
            continue;
        };
        let Some(module_rel) =
            super::super::util::module_rel_path_for_output(module, module_id, output, &roots)
        else {
            continue;
        };

        let overlay_dir = match scope {
            EvolveScope::Global => {
                super::super::util::overlay_dir_for_scope(engine, module_id, OverlayScope::Global)
            }
            EvolveScope::Machine => {
                super::super::util::overlay_dir_for_scope(engine, module_id, OverlayScope::Machine)
            }
            EvolveScope::Project => {
                super::super::util::overlay_dir_for_scope(engine, module_id, OverlayScope::Project)
            }
        };

        let dst = overlay_dir.join(&module_rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        write_atomic(&dst, actual).with_context(|| format!("write {}", dst.display()))?;
        touched.push(
            dst.strip_prefix(&engine.repo.repo_dir)
                .unwrap_or(&dst)
                .to_string_lossy()
                .to_string(),
        );
    }

    if touched.is_empty() {
        crate::git::git_in(repo_dir, &["checkout", original.as_str()]).ok();
        anyhow::bail!("no proposeable files (only multi-module outputs or unknown modules)");
    }

    crate::git::git_in(repo_dir, &["add", "-A"])?;

    let commit = std::process::Command::new("git")
        .current_dir(repo_dir)
        .args(["commit", "-m", "chore(evolve): propose overlay updates"])
        .output();

    let committed = match commit {
        Ok(out) if out.status.success() => true,
        Ok(out) => {
            eprintln!(
                "Warning: git commit failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            false
        }
        Err(err) => {
            eprintln!("Warning: failed to run git commit: {err}");
            false
        }
    };

    if committed {
        crate::git::git_in(repo_dir, &["checkout", original.as_str()]).ok();
    }

    if cli.json {
        let files_posix: Vec<String> = touched.iter().map(|p| p.replace('\\', "/")).collect();
        let envelope = JsonEnvelope::ok(
            "evolve.propose",
            serde_json::json!({
                "created": true,
                "branch": branch,
                "scope": scope,
                "files": touched,
                "files_posix": files_posix,
                "committed": committed,
            }),
        );
        print_json(&envelope)?;
    } else {
        println!("Created proposal branch: {branch}");
        for f in &touched {
            println!("- {f}");
        }
        if !committed {
            println!("Note: commit failed; changes are left on the proposal branch.");
        }
    }

    Ok(())
}
