use std::path::PathBuf;

use anyhow::Context as _;

use super::args::{Cli, OverlayScope};
use crate::config::{Manifest, Module, ModuleType};
use crate::deploy::TargetPath;
use crate::diff::unified_diff;
use crate::engine::Engine;
use crate::overlay::resolve_upstream_module_root;
use crate::targets::TargetRoot;
use crate::user_error::UserError;

pub(crate) const MUTATING_COMMAND_IDS: &[&str] = &[
    "init",
    "import --apply",
    "add",
    "remove",
    "lock",
    "fetch",
    "update",
    "deploy --apply",
    "rollback",
    "bootstrap",
    "doctor --fix",
    "overlay edit",
    "overlay rebase",
    "remote set",
    "sync",
    "record",
    "evolve propose",
    "evolve restore",
    "policy lock",
];

pub(crate) fn require_yes_for_json_mutation(
    cli: &Cli,
    command_id: &'static str,
) -> anyhow::Result<()> {
    debug_assert!(
        MUTATING_COMMAND_IDS.contains(&command_id),
        "mutating command id must be registered in MUTATING_COMMAND_IDS: {command_id}"
    );
    if cli.json && !cli.yes {
        return Err(UserError::confirm_required(command_id));
    }
    Ok(())
}

pub(crate) fn selected_targets(
    manifest: &Manifest,
    target_filter: &str,
) -> anyhow::Result<Vec<String>> {
    crate::target_selection::selected_targets(manifest, target_filter)
}

pub(crate) fn filter_managed(
    managed: crate::deploy::ManagedPaths,
    target_filter: &str,
) -> crate::deploy::ManagedPaths {
    managed
        .into_iter()
        .filter(|tp| target_filter == "all" || tp.target == target_filter)
        .collect()
}

pub(crate) fn best_root_idx(
    roots: &[TargetRoot],
    target: &str,
    path: &std::path::Path,
) -> Option<usize> {
    roots
        .iter()
        .enumerate()
        .filter(|(_, r)| r.target == target)
        .filter(|(_, r)| path.strip_prefix(&r.root).is_ok())
        .max_by_key(|(_, r)| r.root.components().count())
        .map(|(idx, _)| idx)
}

pub(crate) fn overlay_dir_for_scope(
    engine: &Engine,
    module_id: &str,
    scope: OverlayScope,
) -> PathBuf {
    let fs_key = crate::ids::module_fs_key(module_id);
    let canonical = match scope {
        OverlayScope::Global => engine.repo.repo_dir.join("overlays").join(&fs_key),
        OverlayScope::Machine => engine
            .repo
            .repo_dir
            .join("overlays/machines")
            .join(&engine.machine_id)
            .join(&fs_key),
        OverlayScope::Project => engine
            .repo
            .repo_dir
            .join("projects")
            .join(&engine.project.project_id)
            .join("overlays")
            .join(&fs_key),
    };

    let legacy_fs_key = crate::ids::module_fs_key_unbounded(module_id);
    let legacy_fs_key = (legacy_fs_key != fs_key).then(|| match scope {
        OverlayScope::Global => engine.repo.repo_dir.join("overlays").join(&legacy_fs_key),
        OverlayScope::Machine => engine
            .repo
            .repo_dir
            .join("overlays/machines")
            .join(&engine.machine_id)
            .join(&legacy_fs_key),
        OverlayScope::Project => engine
            .repo
            .repo_dir
            .join("projects")
            .join(&engine.project.project_id)
            .join("overlays")
            .join(&legacy_fs_key),
    });

    let legacy = crate::ids::is_safe_legacy_path_component(module_id).then(|| match scope {
        OverlayScope::Global => engine.repo.repo_dir.join("overlays").join(module_id),
        OverlayScope::Machine => engine
            .repo
            .repo_dir
            .join("overlays/machines")
            .join(&engine.machine_id)
            .join(module_id),
        OverlayScope::Project => engine
            .repo
            .repo_dir
            .join("projects")
            .join(&engine.project.project_id)
            .join("overlays")
            .join(module_id),
    });

    if canonical.exists() {
        canonical
    } else if legacy_fs_key.as_ref().is_some_and(|p| p.exists()) {
        legacy_fs_key.expect("legacy fs_key exists")
    } else if legacy.as_ref().is_some_and(|p| p.exists()) {
        legacy.expect("legacy exists")
    } else {
        canonical
    }
}

pub(crate) struct ManifestModuleIdsIndex {
    pub index: std::collections::BTreeMap<TargetPath, Vec<String>>,
    pub warnings: Vec<String>,
}

pub(crate) fn load_manifest_module_ids(
    roots: &[TargetRoot],
) -> anyhow::Result<ManifestModuleIdsIndex> {
    let mut out = std::collections::BTreeMap::new();
    let mut warnings: Vec<String> = Vec::new();
    for root in roots {
        let preferred = crate::target_manifest::manifest_path_for_target(&root.root, &root.target);
        let legacy = crate::target_manifest::legacy_manifest_path(&root.root);

        let (path, used_legacy) = if preferred.exists() {
            (preferred, false)
        } else if legacy.exists() {
            (legacy, true)
        } else {
            continue;
        };

        if used_legacy {
            warnings.push(format!(
                "target manifest ({}): using legacy manifest filename {} (consider running `agentpack deploy --apply` to migrate)",
                root.target,
                path.display(),
            ));
        }

        let (manifest, manifest_warnings) =
            crate::target_manifest::read_target_manifest_soft(&path, &root.target);
        warnings.extend(manifest_warnings);
        let Some(manifest) = manifest else {
            continue;
        };
        for f in manifest.managed_files {
            if std::path::Path::new(&f.path).is_absolute() {
                warnings.push(format!(
                    "target manifest ({}): skipped invalid entry path {:?} in {}",
                    root.target,
                    f.path,
                    path.display()
                ));
                continue;
            }
            if std::path::Path::new(&f.path)
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                warnings.push(format!(
                    "target manifest ({}): skipped invalid entry path {:?} in {}",
                    root.target,
                    f.path,
                    path.display()
                ));
                continue;
            }
            out.insert(
                TargetPath {
                    target: root.target.clone(),
                    path: root.root.join(&f.path),
                },
                f.module_ids,
            );
        }
    }
    Ok(ManifestModuleIdsIndex {
        index: out,
        warnings,
    })
}

pub(crate) fn module_rel_path_for_output(
    module: &Module,
    module_id: &str,
    output: &TargetPath,
    roots: &[TargetRoot],
) -> Option<String> {
    match module.module_type {
        ModuleType::Instructions => Some("AGENTS.md".to_string()),
        ModuleType::Prompt | ModuleType::Command => output
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string()),
        ModuleType::Skill => {
            let best = crate::targets::best_root_for(roots, &output.target, &output.path)?;
            let rel = output.path.strip_prefix(&best.root).ok()?;
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let skill_name = module_name_from_id(module_id);
            let Some((first, rest)) = rel_str.split_once('/') else {
                return Some(rel_str);
            };
            if first == skill_name && !rest.is_empty() {
                Some(rest.to_string())
            } else {
                Some(rel_str)
            }
        }
    }
}

pub(crate) fn source_layer_for_module_file(
    engine: &Engine,
    module: &Module,
    module_rel_path: &str,
) -> anyhow::Result<String> {
    let rel = std::path::Path::new(module_rel_path);

    let global = overlay_dir_for_scope(engine, &module.id, OverlayScope::Global);
    let machine = overlay_dir_for_scope(engine, &module.id, OverlayScope::Machine);
    let project = overlay_dir_for_scope(engine, &module.id, OverlayScope::Project);

    if project.join(rel).exists() {
        return Ok("project".to_string());
    }
    if machine.join(rel).exists() {
        return Ok("machine".to_string());
    }
    if global.join(rel).exists() {
        return Ok("global".to_string());
    }

    let upstream = resolve_upstream_module_root(&engine.home, &engine.repo, module)?;
    if upstream.join(rel).exists() {
        return Ok("upstream".to_string());
    }

    Ok("missing".to_string())
}

pub(crate) fn module_name_from_id(module_id: &str) -> String {
    module_id
        .split_once(':')
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| crate::store::sanitize_module_id(module_id))
}

pub(crate) fn print_diff(
    plan: &crate::deploy::PlanResult,
    desired: &crate::deploy::DesiredState,
) -> anyhow::Result<()> {
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

        println!("\n=== {} {} ===", c.target, c.path);
        match (before_text, after_text) {
            (Some(from), Some(to)) => {
                print!(
                    "{}",
                    unified_diff(
                        &from,
                        &to,
                        &format!("before: {}", c.path),
                        &format!("after: {}", c.path)
                    )
                );
            }
            _ => {
                println!("(binary or non-utf8 content; diff omitted)");
            }
        }
    }

    Ok(())
}

pub(crate) fn confirm(prompt: &str) -> anyhow::Result<bool> {
    use std::io::Write as _;

    print!("{prompt} [y/N] ");
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let s = input.trim().to_lowercase();
    Ok(s == "y" || s == "yes")
}

pub(crate) fn expand_tilde(s: &str) -> anyhow::Result<PathBuf> {
    if let Some(rest) = s.strip_prefix("~/") {
        let home = dirs::home_dir().context("resolve home dir")?;
        return Ok(home.join(rest));
    }
    Ok(PathBuf::from(s))
}

pub(crate) fn codex_home_for_manifest(manifest: &Manifest) -> anyhow::Result<PathBuf> {
    if let Some(cfg) = manifest.targets.get("codex") {
        if let Some(serde_yaml::Value::String(s)) = cfg.options.get("codex_home") {
            if !s.trim().is_empty() {
                return expand_tilde(s);
            }
        }
    }

    if let Ok(env) = std::env::var("CODEX_HOME") {
        if !env.trim().is_empty() {
            return expand_tilde(&env);
        }
    }

    expand_tilde("~/.codex")
}
