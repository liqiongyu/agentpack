use anyhow::Context as _;

use crate::engine::Engine;
use crate::fs::write_atomic;
use crate::output::{JsonEnvelope, print_json};
use crate::project::ProjectContext;

use super::super::args::BootstrapScope;
use super::Ctx;

fn ensure_gitignore_contains(repo_root: &std::path::Path, line: &str) -> anyhow::Result<bool> {
    let gitignore_path = repo_root.join(".gitignore");
    let mut contents = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
    let already = contents.lines().any(|l| l.trim() == line);
    if already {
        return Ok(false);
    }

    if !contents.is_empty() && !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(line);
    contents.push('\n');
    write_atomic(&gitignore_path, contents.as_bytes())
        .with_context(|| format!("write {}", gitignore_path.display()))?;
    Ok(true)
}

pub(crate) fn run(ctx: &Ctx<'_>, git: bool, bootstrap: bool) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "init")?;

    ctx.repo.init_repo_skeleton().context("init repo")?;

    let mut gitignore_updated = false;
    if git {
        crate::git::git_in(&ctx.repo.repo_dir, &["init"]).context("git init")?;
        gitignore_updated |=
            ensure_gitignore_contains(&ctx.repo.repo_dir, ".agentpack.manifest.json")?;
        gitignore_updated |= ensure_gitignore_contains(&ctx.repo.repo_dir, ".DS_Store")?;
    }

    let mut bootstrap_result: Option<serde_json::Value> = None;
    if bootstrap {
        let mut engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
        engine.project =
            ProjectContext::detect(&ctx.repo.repo_dir).context("detect project (repo dir)")?;

        let targets = super::super::util::selected_targets(&engine.manifest, &ctx.cli.target)?;
        let (desired, roots, scope_str) =
            super::bootstrap::build_desired_and_roots(&engine, &targets, BootstrapScope::Project)?;
        let plan = crate::deploy::plan(&desired, None)?;

        if ctx.cli.dry_run {
            bootstrap_result = Some(serde_json::json!({
                "applied": false,
                "reason": "dry_run",
                "targets": targets,
                "scope": scope_str,
                "changes": plan.changes,
                "summary": plan.summary,
            }));
        } else if plan.changes.is_empty() {
            bootstrap_result = Some(serde_json::json!({
                "applied": false,
                "reason": "no_changes",
                "targets": targets,
                "scope": scope_str,
                "changes": plan.changes,
                "summary": plan.summary,
            }));
        } else {
            let snapshot =
                crate::apply::apply_plan(&engine.home, "bootstrap", &plan, &desired, None, &roots)?;
            bootstrap_result = Some(serde_json::json!({
                "applied": true,
                "snapshot_id": snapshot.id,
                "targets": targets,
                "scope": scope_str,
                "changes": plan.changes,
                "summary": plan.summary,
            }));
        }
    }

    if ctx.cli.json {
        let mut data = serde_json::json!({
            "repo": ctx.repo.repo_dir,
            "repo_posix": crate::paths::path_to_posix_string(&ctx.repo.repo_dir),
        });
        if git {
            data.as_object_mut()
                .context("init json data must be an object")?
                .insert(
                "git".to_string(),
                serde_json::json!({
                    "initialized": true,
                    "gitignore_updated": gitignore_updated,
                    "gitignore_path": ctx.repo.repo_dir.join(".gitignore"),
                    "gitignore_path_posix": crate::paths::path_to_posix_string(&ctx.repo.repo_dir.join(".gitignore")),
                }),
            );
        }

        if let Some(bootstrap_result) = bootstrap_result {
            data.as_object_mut()
                .context("init json data must be an object")?
                .insert("bootstrap".to_string(), bootstrap_result);
        }

        let envelope = JsonEnvelope::ok("init", data);
        print_json(&envelope)?;
    } else {
        println!(
            "Initialized agentpack repo at {}",
            ctx.repo.repo_dir.display()
        );
        if git {
            println!("Initialized git repo and ensured .gitignore (updated={gitignore_updated})");
        }
        if let Some(v) = bootstrap_result {
            let applied = v["applied"].as_bool().unwrap_or(false);
            if applied {
                println!(
                    "Bootstrapped operator assets (snapshot={})",
                    v["snapshot_id"].as_str().unwrap_or_default()
                );
            } else {
                println!(
                    "Bootstrap skipped (reason={})",
                    v["reason"].as_str().unwrap_or_default()
                );
            }
        }
    }

    Ok(())
}
