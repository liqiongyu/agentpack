use std::collections::BTreeMap;
use std::io::IsTerminal as _;

use anyhow::Context as _;

use crate::config::{Manifest, Profile, TargetConfig, TargetMode, TargetScope};
use crate::engine::Engine;
use crate::fs::write_atomic;
use crate::output::{JsonEnvelope, print_json};
use crate::project::ProjectContext;
use crate::user_error::UserError;

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

fn prompt_line(prompt: &str) -> anyhow::Result<String> {
    use std::io::Write as _;
    eprint!("{prompt}");
    std::io::stderr().flush().context("flush stderr")?;

    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read stdin")?;
    Ok(line.trim().to_string())
}

fn prompt_yes_no(prompt: &str, default: bool) -> anyhow::Result<bool> {
    loop {
        let suffix = if default { "[Y/n]" } else { "[y/N]" };
        let v = prompt_line(&format!("{prompt} {suffix} "))?;
        let v = v.trim().to_ascii_lowercase();
        if v.is_empty() {
            return Ok(default);
        }
        match v.as_str() {
            "y" | "yes" | "true" | "1" => return Ok(true),
            "n" | "no" | "false" | "0" => return Ok(false),
            _ => eprintln!("Invalid input: {v}"),
        }
    }
}

fn prompt_scope() -> anyhow::Result<TargetScope> {
    loop {
        let v = prompt_line("Scope (project|both) [both]: ")?;
        let v = v.trim().to_ascii_lowercase();
        if v.is_empty() || v == "both" {
            return Ok(TargetScope::Both);
        }
        if v == "project" {
            return Ok(TargetScope::Project);
        }
        eprintln!("Invalid scope: {v}");
    }
}

fn prompt_targets() -> anyhow::Result<Vec<String>> {
    let mut default = Vec::new();
    if crate::target_registry::is_compiled_target("codex") {
        default.push("codex".to_string());
    }
    if crate::target_registry::is_compiled_target("claude_code") {
        default.push("claude_code".to_string());
    }
    if default.is_empty() {
        default = crate::target_registry::COMPILED_TARGETS
            .iter()
            .map(|t| (*t).to_string())
            .collect();
    }
    if default.is_empty() {
        anyhow::bail!("no targets compiled into this agentpack binary");
    }

    default.sort();
    default.dedup();
    let default_display = default.join(",");
    loop {
        let v = prompt_line(&format!("Targets (comma-separated) [{default_display}]: "))?;
        let mut targets = if v.trim().is_empty() {
            default.clone()
        } else {
            v.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        };

        targets.sort();
        targets.dedup();

        let invalid = targets
            .iter()
            .filter(|t| !crate::target_registry::is_compiled_target(t))
            .cloned()
            .collect::<Vec<_>>();
        if invalid.is_empty() && !targets.is_empty() {
            return Ok(targets);
        }

        if targets.is_empty() {
            eprintln!("Select at least one target.");
        } else {
            eprintln!(
                "Unsupported target(s): {} (allowed: {})",
                invalid.join(", "),
                crate::target_registry::COMPILED_TARGETS.join(", ")
            );
        }
    }
}

fn guided_manifest(targets: &[String], scope: TargetScope) -> (Manifest, Vec<String>) {
    let mut warnings = Vec::new();

    let mut profiles = BTreeMap::new();
    profiles.insert(
        "default".to_string(),
        Profile {
            include_tags: vec!["base".to_string()],
            include_modules: Vec::new(),
            exclude_modules: Vec::new(),
        },
    );

    let mut out_targets: BTreeMap<String, TargetConfig> = BTreeMap::new();
    for target in targets {
        match target.as_str() {
            "codex" => {
                let mut options = BTreeMap::new();
                options.insert(
                    "codex_home".to_string(),
                    serde_yaml::Value::String("~/.codex".to_string()),
                );
                options.insert(
                    "write_repo_skills".to_string(),
                    serde_yaml::Value::Bool(true),
                );
                options.insert(
                    "write_user_skills".to_string(),
                    serde_yaml::Value::Bool(true),
                );
                options.insert(
                    "write_user_prompts".to_string(),
                    serde_yaml::Value::Bool(true),
                );
                options.insert(
                    "write_agents_global".to_string(),
                    serde_yaml::Value::Bool(true),
                );
                options.insert(
                    "write_agents_repo_root".to_string(),
                    serde_yaml::Value::Bool(true),
                );

                out_targets.insert(
                    "codex".to_string(),
                    TargetConfig {
                        mode: TargetMode::Files,
                        scope: scope.clone(),
                        options,
                    },
                );
            }
            "claude_code" => {
                let mut options = BTreeMap::new();
                options.insert(
                    "write_repo_commands".to_string(),
                    serde_yaml::Value::Bool(true),
                );
                options.insert(
                    "write_user_commands".to_string(),
                    serde_yaml::Value::Bool(true),
                );
                options.insert(
                    "write_repo_skills".to_string(),
                    serde_yaml::Value::Bool(false),
                );
                options.insert(
                    "write_user_skills".to_string(),
                    serde_yaml::Value::Bool(false),
                );

                out_targets.insert(
                    "claude_code".to_string(),
                    TargetConfig {
                        mode: TargetMode::Files,
                        scope: scope.clone(),
                        options,
                    },
                );
            }
            "cursor" => {
                if matches!(scope, TargetScope::Both | TargetScope::User) {
                    warnings.push(
                        "cursor supports project scope only; using scope=project".to_string(),
                    );
                }

                let mut options = BTreeMap::new();
                options.insert("write_rules".to_string(), serde_yaml::Value::Bool(true));

                out_targets.insert(
                    "cursor".to_string(),
                    TargetConfig {
                        mode: TargetMode::Files,
                        scope: TargetScope::Project,
                        options,
                    },
                );
            }
            "vscode" => {
                if matches!(scope, TargetScope::Both | TargetScope::User) {
                    warnings.push(
                        "vscode supports project scope only; using scope=project".to_string(),
                    );
                }

                let mut options = BTreeMap::new();
                options.insert(
                    "write_instructions".to_string(),
                    serde_yaml::Value::Bool(true),
                );
                options.insert("write_prompts".to_string(), serde_yaml::Value::Bool(true));

                out_targets.insert(
                    "vscode".to_string(),
                    TargetConfig {
                        mode: TargetMode::Files,
                        scope: TargetScope::Project,
                        options,
                    },
                );
            }
            "jetbrains" => {
                if matches!(scope, TargetScope::Both | TargetScope::User) {
                    warnings.push(
                        "jetbrains supports project scope only; using scope=project".to_string(),
                    );
                }

                let mut options = BTreeMap::new();
                options.insert(
                    "write_guidelines".to_string(),
                    serde_yaml::Value::Bool(true),
                );

                out_targets.insert(
                    "jetbrains".to_string(),
                    TargetConfig {
                        mode: TargetMode::Files,
                        scope: TargetScope::Project,
                        options,
                    },
                );
            }
            _ => {}
        }
    }

    (
        Manifest {
            version: 1,
            profiles,
            targets: out_targets,
            modules: Vec::new(),
        },
        warnings,
    )
}

pub(crate) fn run(ctx: &Ctx<'_>, guided: bool, git: bool, bootstrap: bool) -> anyhow::Result<()> {
    if guided {
        let stdin_is_terminal = std::io::stdin().is_terminal();
        let stdout_is_terminal = std::io::stdout().is_terminal();
        if !stdin_is_terminal || !stdout_is_terminal {
            if ctx.cli.json {
                return Err(anyhow::Error::new(
                    UserError::new("E_TTY_REQUIRED", "init --guided requires a TTY").with_details(
                        serde_json::json!({
                            "stdin_is_terminal": stdin_is_terminal,
                            "stdout_is_terminal": stdout_is_terminal,
                            "hint": "run init --guided in an interactive terminal",
                        }),
                    ),
                ));
            }
            anyhow::bail!("init --guided requires a TTY (stdin and stdout must be terminals)");
        }

        if ctx.repo.manifest_path.exists() {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!(
                        "config manifest already exists: {}",
                        ctx.repo.manifest_path.display()
                    ),
                )
                .with_details(serde_json::json!({
                    "path": ctx.repo.manifest_path,
                    "path_posix": crate::paths::path_to_posix_string(&ctx.repo.manifest_path),
                    "hint": "refusing to overwrite; edit the existing manifest or remove it and re-run init",
                })),
            ));
        }

        // If the caller wants machine-readable output, require explicit confirmation.
        super::super::util::require_yes_for_json_mutation(ctx.cli, "init")?;

        let targets = prompt_targets()?;
        let scope = prompt_scope()?;
        let want_bootstrap = if bootstrap {
            true
        } else {
            prompt_yes_no("Bootstrap operator assets after init?", false)?
        };

        ctx.repo.init_repo_skeleton().context("init repo")?;

        let (manifest, mut guided_warnings) = guided_manifest(&targets, scope.clone());
        manifest
            .save(&ctx.repo.manifest_path)
            .context("save guided manifest")?;

        let mut gitignore_updated = false;
        if git {
            crate::git::git_in(&ctx.repo.repo_dir, &["init"]).context("git init")?;
            gitignore_updated |=
                ensure_gitignore_contains(&ctx.repo.repo_dir, ".agentpack.manifest.json")?;
            gitignore_updated |= ensure_gitignore_contains(&ctx.repo.repo_dir, ".DS_Store")?;
        }

        let mut bootstrap_result: Option<serde_json::Value> = None;
        if want_bootstrap {
            let mut engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
            engine.project =
                ProjectContext::detect(&ctx.repo.repo_dir).context("detect project (repo dir)")?;

            let targets = super::super::util::selected_targets(&engine.manifest, &ctx.cli.target)?;
            let (desired, roots, scope_str) = super::bootstrap::build_desired_and_roots(
                &engine,
                &targets,
                BootstrapScope::Project,
            )?;
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
                let snapshot = crate::apply::apply_plan(
                    &engine.home,
                    "bootstrap",
                    &plan,
                    &desired,
                    None,
                    &roots,
                )?;
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
                "guided": true,
                "repo": ctx.repo.repo_dir,
                "repo_posix": crate::paths::path_to_posix_string(&ctx.repo.repo_dir),
                "targets": targets,
                "scope": match scope {
                    TargetScope::Project => "project",
                    TargetScope::Both => "both",
                    TargetScope::User => "user",
                },
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

            let mut envelope = JsonEnvelope::ok("init", data);
            envelope.warnings.append(&mut guided_warnings);
            print_json(&envelope)?;
        } else {
            println!(
                "Initialized agentpack repo at {}",
                ctx.repo.repo_dir.display()
            );
            for w in guided_warnings {
                eprintln!("Warning: {w}");
            }
            if git {
                println!(
                    "Initialized git repo and ensured .gitignore (updated={gitignore_updated})"
                );
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

        return Ok(());
    }

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
