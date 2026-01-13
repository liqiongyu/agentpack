use anyhow::Context as _;

use crate::config::TargetScope;
use crate::deploy::TargetPath;
use crate::engine::Engine;
use crate::hash::sha256_hex;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

#[derive(Default)]
struct NextActions {
    human: std::collections::BTreeSet<String>,
    json: std::collections::BTreeSet<String>,
}

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct DriftItem {
        target: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        root: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        root_posix: Option<String>,
        path: String,
        path_posix: String,
        expected: Option<String>,
        actual: Option<String>,
        kind: String,
    }

    #[derive(Default, serde::Serialize)]
    struct DriftSummary {
        modified: u64,
        missing: u64,
        extra: u64,
    }

    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let targets = super::super::util::selected_targets(&engine.manifest, &ctx.cli.target)?;
    let render = engine.desired_state(&ctx.cli.profile, &ctx.cli.target)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;
    let mut next_actions = NextActions::default();
    warn_operator_assets_if_outdated(&engine, ctx.cli, &targets, &mut warnings, &mut next_actions)?;
    let prefix = action_prefix(ctx.cli);

    let mut drift = Vec::new();
    let mut summary = DriftSummary::default();

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
        next_actions
            .human
            .insert(format!("{prefix} deploy --apply"));
        next_actions
            .json
            .insert(format!("{prefix} deploy --apply --yes --json"));
        for (tp, desired_file) in &desired {
            let expected = format!("sha256:{}", sha256_hex(&desired_file.bytes));
            let root = super::super::util::best_root_idx(&roots, &tp.target, &tp.path)
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
                            root_posix: root.map(crate::paths::path_to_posix_string),
                            path: tp.path.to_string_lossy().to_string(),
                            path_posix: crate::paths::path_to_posix_string(&tp.path),
                            expected: Some(expected),
                            actual: Some(actual),
                            kind: "modified".to_string(),
                        });
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    summary.missing += 1;
                    drift.push(DriftItem {
                        target: tp.target.clone(),
                        root: root.map(|p| p.to_string_lossy().to_string()),
                        root_posix: root.map(crate::paths::path_to_posix_string),
                        path: tp.path.to_string_lossy().to_string(),
                        path_posix: crate::paths::path_to_posix_string(&tp.path),
                        expected: Some(expected),
                        actual: None,
                        kind: "missing".to_string(),
                    });
                }
                Err(err) => return Err(err).context("read deployed file"),
            }
        }
    } else {
        let mut desired_by_root: Vec<Vec<(&TargetPath, &crate::deploy::DesiredFile)>> =
            vec![Vec::new(); roots.len()];
        for (tp, desired_file) in &desired {
            let Some(root_idx) = super::super::util::best_root_idx(&roots, &tp.target, &tp.path)
            else {
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
                next_actions
                    .human
                    .insert(format!("{prefix} deploy --apply"));
                next_actions
                    .json
                    .insert(format!("{prefix} deploy --apply --yes --json"));
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
                                    root_posix: Some(crate::paths::path_to_posix_string(
                                        &root.root,
                                    )),
                                    path: tp.path.to_string_lossy().to_string(),
                                    path_posix: crate::paths::path_to_posix_string(&tp.path),
                                    expected: Some(expected),
                                    actual: Some(actual),
                                    kind: "modified".to_string(),
                                });
                            }
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                            summary.missing += 1;
                            drift.push(DriftItem {
                                target: tp.target.clone(),
                                root: Some(root.root.to_string_lossy().to_string()),
                                root_posix: Some(crate::paths::path_to_posix_string(&root.root)),
                                path: tp.path.to_string_lossy().to_string(),
                                path_posix: crate::paths::path_to_posix_string(&tp.path),
                                expected: Some(expected),
                                actual: None,
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
                                    root_posix: Some(crate::paths::path_to_posix_string(
                                        &root.root,
                                    )),
                                    path: tp.path.to_string_lossy().to_string(),
                                    path_posix: crate::paths::path_to_posix_string(&tp.path),
                                    expected: Some(exp.clone()),
                                    actual: Some(actual),
                                    kind: "modified".to_string(),
                                });
                            }
                        } else {
                            summary.extra += 1;
                            drift.push(DriftItem {
                                target: tp.target.clone(),
                                root: Some(root.root.to_string_lossy().to_string()),
                                root_posix: Some(crate::paths::path_to_posix_string(&root.root)),
                                path: tp.path.to_string_lossy().to_string(),
                                path_posix: crate::paths::path_to_posix_string(&tp.path),
                                expected: None,
                                actual: Some(actual),
                                kind: "extra".to_string(),
                            });
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                        if let Some(exp) = expected {
                            summary.missing += 1;
                            drift.push(DriftItem {
                                target: tp.target.clone(),
                                root: Some(root.root.to_string_lossy().to_string()),
                                root_posix: Some(crate::paths::path_to_posix_string(&root.root)),
                                path: tp.path.to_string_lossy().to_string(),
                                path_posix: crate::paths::path_to_posix_string(&tp.path),
                                expected: Some(exp),
                                actual: None,
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
                    root_posix: Some(crate::paths::path_to_posix_string(&root.root)),
                    path: tp.path.to_string_lossy().to_string(),
                    path_posix: crate::paths::path_to_posix_string(&tp.path),
                    expected: None,
                    actual: Some(format!("sha256:{}", sha256_hex(&std::fs::read(&tp.path)?))),
                    kind: "extra".to_string(),
                });
            }
        }
    }

    if summary.modified > 0 || summary.missing > 0 {
        next_actions
            .human
            .insert(format!("{prefix} preview --diff"));
        next_actions
            .human
            .insert(format!("{prefix} deploy --apply"));
        next_actions
            .human
            .insert(format!("{prefix} evolve propose"));

        next_actions
            .json
            .insert(format!("{prefix} preview --diff --json"));
        next_actions
            .json
            .insert(format!("{prefix} deploy --apply --yes --json"));
        next_actions
            .json
            .insert(format!("{prefix} evolve propose --yes --json"));
    } else if summary.extra > 0 {
        next_actions
            .human
            .insert(format!("{prefix} preview --diff"));
        next_actions
            .json
            .insert(format!("{prefix} preview --diff --json"));
    }

    if ctx.cli.json {
        let mut data = serde_json::json!({
            "profile": ctx.cli.profile,
            "targets": targets,
            "drift": drift,
            "summary": summary,
        });
        if !next_actions.json.is_empty() {
            data.as_object_mut()
                .context("status json data must be an object")?
                .insert(
                    "next_actions".to_string(),
                    serde_json::to_value(&next_actions.json).context("serialize next_actions")?,
                );
        }

        let mut envelope = JsonEnvelope::ok("status", data);
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else if drift.is_empty() {
        for w in warnings {
            eprintln!("Warning: {w}");
        }
        println!("No drift");

        if !next_actions.human.is_empty() {
            println!();
            println!("Next actions:");
            for action in next_actions.human {
                println!("- {action}");
            }
        }
    } else {
        for w in warnings {
            eprintln!("Warning: {w}");
        }
        println!("Drift ({}):", drift.len());
        println!(
            "Summary: modified={} missing={} extra={}",
            summary.modified, summary.missing, summary.extra
        );
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
        let mut last_group: Option<(String, String)> = None;
        for d in drift {
            let root = d.root.as_deref().unwrap_or("<unknown>");
            let group = (d.target.clone(), root.to_string());
            if last_group.as_ref() != Some(&group) {
                println!("Root: {} ({})", root, d.target);
                last_group = Some(group);
            }
            println!("- {} {}", d.kind, d.path);
        }

        if !next_actions.human.is_empty() {
            println!();
            println!("Next actions:");
            for action in next_actions.human {
                println!("- {action}");
            }
        }
    }

    Ok(())
}

fn target_scope_flags(scope: &TargetScope) -> (bool, bool) {
    match scope {
        TargetScope::User => (true, false),
        TargetScope::Project => (false, true),
        TargetScope::Both => (true, true),
    }
}

fn extract_agentpack_version(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some((_, rest)) = line.split_once("agentpack_version:") {
            let mut value = rest.trim();
            value = value.trim_end_matches("-->");
            value = value.trim();
            value = value.trim_matches(|c| c == '"' || c == '\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn warn_operator_assets_if_outdated(
    engine: &Engine,
    cli: &crate::cli::args::Cli,
    targets: &[String],
    warnings: &mut Vec<String>,
    next_actions: &mut NextActions,
) -> anyhow::Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    for target in targets {
        match target.as_str() {
            "codex" => {
                let Some(cfg) = engine.manifest.targets.get("codex") else {
                    continue;
                };
                let (allow_user, allow_project) = target_scope_flags(&cfg.scope);
                let codex_home = super::super::util::codex_home_for_manifest(&engine.manifest)?;

                if allow_user {
                    let path = codex_home.join("skills/agentpack-operator/SKILL.md");
                    check_operator_file(
                        &path,
                        "codex/user",
                        current,
                        warnings,
                        &bootstrap_action(cli, "codex", "user"),
                        next_actions,
                    )?;
                }
                if allow_project {
                    let path = engine
                        .project
                        .project_root
                        .join(".codex/skills/agentpack-operator/SKILL.md");
                    check_operator_file(
                        &path,
                        "codex/project",
                        current,
                        warnings,
                        &bootstrap_action(cli, "codex", "project"),
                        next_actions,
                    )?;
                }
            }
            "claude_code" => {
                let Some(cfg) = engine.manifest.targets.get("claude_code") else {
                    continue;
                };
                let (allow_user, allow_project) = target_scope_flags(&cfg.scope);

                if allow_user {
                    let dir = super::super::util::expand_tilde("~/.claude/commands")?;
                    check_operator_command_dir(
                        &dir,
                        "claude_code/user",
                        current,
                        warnings,
                        &bootstrap_action(cli, "claude_code", "user"),
                        next_actions,
                    )?;
                }
                if allow_project {
                    let dir = engine.project.project_root.join(".claude/commands");
                    check_operator_command_dir(
                        &dir,
                        "claude_code/project",
                        current,
                        warnings,
                        &bootstrap_action(cli, "claude_code", "project"),
                        next_actions,
                    )?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn check_operator_file(
    path: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
    next_actions: &mut NextActions,
) -> anyhow::Result<()> {
    if !path.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            path.display()
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
        return Ok(());
    }

    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read operator asset {}", path.display()))?;
    let Some(have) = extract_agentpack_version(&text) else {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
        return Ok(());
    };

    if have != current {
        warnings.push(format!(
            "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
            path.display(),
            have,
            current
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
    }

    Ok(())
}

fn check_operator_command_dir(
    dir: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
    next_actions: &mut NextActions,
) -> anyhow::Result<()> {
    if !dir.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            dir.display()
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
        return Ok(());
    }

    let mut missing = Vec::new();
    let mut missing_version = None;
    for name in [
        "ap-doctor.md",
        "ap-update.md",
        "ap-preview.md",
        "ap-plan.md",
        "ap-deploy.md",
        "ap-status.md",
        "ap-diff.md",
        "ap-explain.md",
        "ap-evolve.md",
    ] {
        let path = dir.join(name);
        if !path.exists() {
            missing.push(name.to_string());
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("read operator asset {}", path.display()))?;
        let Some(have) = extract_agentpack_version(&text) else {
            missing_version = Some(path);
            continue;
        };
        if have != current {
            warnings.push(format!(
                "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
                path.display(),
                have,
                current
            ));
        }
    }

    if let Some(path) = missing_version {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
        return Ok(());
    }

    if !missing.is_empty() {
        warnings.push(format!(
            "operator assets incomplete ({location}): missing {}; run: {suggested}",
            missing.join(", "),
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
    }

    Ok(())
}

fn bootstrap_action(cli: &crate::cli::args::Cli, target: &str, scope: &str) -> String {
    let mut out = String::from("agentpack");
    if let Some(repo) = &cli.repo {
        out.push_str(&format!(" --repo {}", repo.display()));
    }
    if let Some(machine) = &cli.machine {
        out.push_str(&format!(" --machine {machine}"));
    }
    out.push_str(&format!(" --target {target} bootstrap --scope {scope}"));
    out
}

fn action_prefix(cli: &crate::cli::args::Cli) -> String {
    let mut out = String::from("agentpack");
    if let Some(repo) = &cli.repo {
        out.push_str(&format!(" --repo {}", repo.display()));
    }
    if cli.profile != "default" {
        out.push_str(&format!(" --profile {}", cli.profile));
    }
    if cli.target != "all" {
        out.push_str(&format!(" --target {}", cli.target));
    }
    if let Some(machine) = &cli.machine {
        out.push_str(&format!(" --machine {machine}"));
    }
    out
}
