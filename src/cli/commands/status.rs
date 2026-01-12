use anyhow::Context as _;

use crate::config::TargetScope;
use crate::deploy::TargetPath;
use crate::engine::Engine;
use crate::hash::sha256_hex;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct DriftItem {
        target: String,
        path: String,
        path_posix: String,
        expected: Option<String>,
        actual: Option<String>,
        kind: String,
    }

    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let targets = super::super::util::selected_targets(&engine.manifest, &ctx.cli.target)?;
    let render = engine.desired_state(&ctx.cli.profile, &ctx.cli.target)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;
    warn_operator_assets_if_outdated(&engine, &targets, &mut warnings)?;
    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
    let managed_paths_from_manifest =
        super::super::util::filter_managed(managed_paths_from_manifest, &ctx.cli.target);

    let mut drift = Vec::new();
    if managed_paths_from_manifest.is_empty() {
        warnings.push(
            "no target manifests found; drift may be inaccurate (run deploy --apply to write manifests)"
                .to_string(),
        );
        for (tp, desired_file) in &desired {
            let expected = format!("sha256:{}", sha256_hex(&desired_file.bytes));
            match std::fs::read(&tp.path) {
                Ok(actual_bytes) => {
                    let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                    if actual != expected {
                        drift.push(DriftItem {
                            target: tp.target.clone(),
                            path: tp.path.to_string_lossy().to_string(),
                            path_posix: crate::paths::path_to_posix_string(&tp.path),
                            expected: Some(expected),
                            actual: Some(actual),
                            kind: "modified".to_string(),
                        });
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => drift.push(DriftItem {
                    target: tp.target.clone(),
                    path: tp.path.to_string_lossy().to_string(),
                    path_posix: crate::paths::path_to_posix_string(&tp.path),
                    expected: Some(expected),
                    actual: None,
                    kind: "missing".to_string(),
                }),
                Err(err) => return Err(err).context("read deployed file"),
            }
        }
    } else {
        for tp in &managed_paths_from_manifest {
            let expected = desired
                .get(tp)
                .map(|f| format!("sha256:{}", sha256_hex(&f.bytes)));
            match std::fs::read(&tp.path) {
                Ok(actual_bytes) => {
                    let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                    if let Some(exp) = &expected {
                        if &actual != exp {
                            drift.push(DriftItem {
                                target: tp.target.clone(),
                                path: tp.path.to_string_lossy().to_string(),
                                path_posix: crate::paths::path_to_posix_string(&tp.path),
                                expected: Some(exp.clone()),
                                actual: Some(actual),
                                kind: "modified".to_string(),
                            });
                        }
                    } else {
                        drift.push(DriftItem {
                            target: tp.target.clone(),
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
                        drift.push(DriftItem {
                            target: tp.target.clone(),
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

        for root in &roots {
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
                if managed_paths_from_manifest.contains(&tp) {
                    continue;
                }

                drift.push(DriftItem {
                    target: tp.target.clone(),
                    path: tp.path.to_string_lossy().to_string(),
                    path_posix: crate::paths::path_to_posix_string(&tp.path),
                    expected: None,
                    actual: Some(format!("sha256:{}", sha256_hex(&std::fs::read(&tp.path)?))),
                    kind: "extra".to_string(),
                });
            }
        }
    }

    if ctx.cli.json {
        let mut envelope = JsonEnvelope::ok(
            "status",
            serde_json::json!({
                "profile": ctx.cli.profile,
                "targets": targets,
                "drift": drift,
            }),
        );
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else if drift.is_empty() {
        for w in warnings {
            eprintln!("Warning: {w}");
        }
        println!("No drift");
    } else {
        for w in warnings {
            eprintln!("Warning: {w}");
        }
        println!("Drift ({}):", drift.len());
        for d in drift {
            println!("{} {} {}", d.kind, d.target, d.path);
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
    targets: &[String],
    warnings: &mut Vec<String>,
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
                        "agentpack bootstrap --target codex --scope user",
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
                        "agentpack bootstrap --target codex --scope project",
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
                        "agentpack bootstrap --target claude_code --scope user",
                    )?;
                }
                if allow_project {
                    let dir = engine.project.project_root.join(".claude/commands");
                    check_operator_command_dir(
                        &dir,
                        "claude_code/project",
                        current,
                        warnings,
                        "agentpack bootstrap --target claude_code --scope project",
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
) -> anyhow::Result<()> {
    if !path.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            path.display()
        ));
        return Ok(());
    }

    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read operator asset {}", path.display()))?;
    let Some(have) = extract_agentpack_version(&text) else {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        return Ok(());
    };

    if have != current {
        warnings.push(format!(
            "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
            path.display(),
            have,
            current
        ));
    }

    Ok(())
}

fn check_operator_command_dir(
    dir: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
) -> anyhow::Result<()> {
    if !dir.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            dir.display()
        ));
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
        return Ok(());
    }

    if !missing.is_empty() {
        warnings.push(format!(
            "operator assets incomplete ({location}): missing {}; run: {suggested}",
            missing.join(", "),
        ));
    }

    Ok(())
}
