use anyhow::Context as _;

use crate::engine::Engine;
use crate::fs::write_atomic;

#[derive(serde::Serialize)]
pub(crate) struct DoctorRootCheck {
    pub(crate) target: String,
    pub(crate) root: String,
    pub(crate) root_posix: String,
    pub(crate) exists: bool,
    pub(crate) writable: bool,
    pub(crate) scan_extras: bool,
    pub(crate) issues: Vec<String>,
    pub(crate) suggestion: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct DoctorGitignoreFix {
    pub(crate) repo_root: String,
    pub(crate) repo_root_posix: String,
    pub(crate) gitignore_path: String,
    pub(crate) gitignore_path_posix: String,
    pub(crate) updated: bool,
}

pub(crate) struct DoctorReport {
    pub(crate) machine_id: String,
    pub(crate) roots: Vec<DoctorRootCheck>,
    pub(crate) gitignore_fixes: Vec<DoctorGitignoreFix>,
    pub(crate) warnings: Vec<String>,
    pub(crate) needs_gitignore_fix: bool,
}

pub(crate) fn doctor_report_in(
    engine: &Engine,
    profile: &str,
    target_filter: &str,
    fix: bool,
) -> anyhow::Result<DoctorReport> {
    let render = engine.desired_state(profile, target_filter)?;
    let mut warnings = render.warnings;
    warnings.extend(overlay_layout_warnings(engine));

    let mut checks = Vec::new();
    let mut repos_to_fix: std::collections::BTreeSet<std::path::PathBuf> =
        std::collections::BTreeSet::new();

    for root in render.roots {
        let mut issues = Vec::new();
        let exists = root.root.exists();
        let is_dir = root.root.is_dir();

        if !exists {
            issues.push("missing".to_string());
        } else if !is_dir {
            issues.push("not_a_directory".to_string());
        }

        let writable = exists && is_dir && dir_is_writable(&root.root);
        if exists && is_dir && !writable {
            issues.push("not_writable".to_string());
        }

        let suggestion = if !exists {
            Some(format!(
                "create directory: mkdir -p {}",
                root.root.display()
            ))
        } else if exists && is_dir && !writable {
            Some("fix permissions (directory not writable)".to_string())
        } else {
            None
        };

        if exists && is_dir {
            if let Some(repo_root) = git_repo_root(&root.root) {
                let manifest_path =
                    crate::target_manifest::manifest_path_for_target(&root.root, &root.target);
                let rel = manifest_path
                    .strip_prefix(&repo_root)
                    .unwrap_or(manifest_path.as_path());
                let ignored = git_is_ignored(&repo_root, rel);
                if !ignored {
                    warnings.push(format!(
                        "target root is in a git repo and `.agentpack.manifest*.json` is not ignored: root={} repo={}; consider adding it to .gitignore (or run `agentpack doctor --fix`)",
                        root.root.display(),
                        repo_root.display(),
                    ));
                    repos_to_fix.insert(repo_root);
                }
            }
        }

        checks.push(DoctorRootCheck {
            target: root.target,
            root: root.root.to_string_lossy().to_string(),
            root_posix: crate::paths::path_to_posix_string(&root.root),
            exists,
            writable,
            scan_extras: root.scan_extras,
            issues,
            suggestion,
        });
    }

    let mut gitignore_fixes: Vec<DoctorGitignoreFix> = Vec::new();
    if fix && !repos_to_fix.is_empty() {
        for repo_root in &repos_to_fix {
            let updated = ensure_gitignore_contains(
                repo_root,
                crate::target_manifest::TARGET_MANIFEST_GITIGNORE_LINE,
            )
            .context("update .gitignore")?;
            gitignore_fixes.push(DoctorGitignoreFix {
                repo_root: repo_root.display().to_string(),
                repo_root_posix: crate::paths::path_to_posix_string(repo_root),
                gitignore_path: repo_root.join(".gitignore").display().to_string(),
                gitignore_path_posix: crate::paths::path_to_posix_string(
                    &repo_root.join(".gitignore"),
                ),
                updated,
            });
        }
    }

    Ok(DoctorReport {
        machine_id: engine.machine_id.clone(),
        roots: checks,
        gitignore_fixes,
        warnings,
        needs_gitignore_fix: !repos_to_fix.is_empty(),
    })
}

fn git_repo_root(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if root.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(root))
    }
}

fn git_is_ignored(repo_root: &std::path::Path, rel: &std::path::Path) -> bool {
    let rel = rel.to_string_lossy().replace('\\', "/");
    let out = std::process::Command::new("git")
        .current_dir(repo_root)
        .args(["check-ignore", "-q", rel.as_str()])
        .output();
    match out {
        Ok(out) if out.status.success() => true,
        Ok(out) if out.status.code() == Some(1) => false,
        _ => false,
    }
}

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

fn dir_is_writable(dir: &std::path::Path) -> bool {
    if !dir.is_dir() {
        return false;
    }

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let test_path = dir.join(format!(".agentpack-write-test-{nanos}"));
    let created = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&test_path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, b"ok\n"))
        .is_ok();

    if created {
        let _ = std::fs::remove_file(&test_path);
    }

    created
}

#[derive(Clone, Copy, Debug)]
enum OverlayScope {
    Global,
    Machine,
    Project,
}

fn overlay_dir_for_scope(
    engine: &Engine,
    module_id: &str,
    scope: OverlayScope,
) -> std::path::PathBuf {
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

fn overlay_layout_warnings(engine: &Engine) -> Vec<String> {
    fn rel(repo_dir: &std::path::Path, path: &std::path::Path) -> String {
        path.strip_prefix(repo_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    fn dirs_for_scope(
        engine: &Engine,
        module_id: &str,
        scope: OverlayScope,
    ) -> (
        std::path::PathBuf,
        Option<std::path::PathBuf>,
        Option<std::path::PathBuf>,
    ) {
        let bounded = crate::ids::module_fs_key(module_id);
        let canonical = match scope {
            OverlayScope::Global => engine.repo.repo_dir.join("overlays").join(&bounded),
            OverlayScope::Machine => engine
                .repo
                .repo_dir
                .join("overlays/machines")
                .join(&engine.machine_id)
                .join(&bounded),
            OverlayScope::Project => engine
                .repo
                .repo_dir
                .join("projects")
                .join(&engine.project.project_id)
                .join("overlays")
                .join(&bounded),
        };

        let legacy_fs_key = crate::ids::module_fs_key_unbounded(module_id);
        let legacy_fs_key = (legacy_fs_key != bounded).then(|| match scope {
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

        (canonical, legacy_fs_key, legacy)
    }

    let mut warnings = Vec::new();
    for module in &engine.manifest.modules {
        let module_id = module.id.as_str();
        for scope in [
            OverlayScope::Global,
            OverlayScope::Machine,
            OverlayScope::Project,
        ] {
            let scope_name = match scope {
                OverlayScope::Global => "global",
                OverlayScope::Machine => "machine",
                OverlayScope::Project => "project",
            };

            let (canonical, legacy_fs_key, legacy) = dirs_for_scope(engine, module_id, scope);

            let mut existing: Vec<std::path::PathBuf> = Vec::new();
            if canonical.exists() {
                existing.push(canonical.clone());
            }
            if let Some(p) = legacy_fs_key.clone() {
                if p.exists() {
                    existing.push(p);
                }
            }
            if let Some(p) = legacy.clone() {
                if p.exists() {
                    existing.push(p);
                }
            }

            if existing.len() >= 2 {
                let found = existing
                    .iter()
                    .map(|p| rel(&engine.repo.repo_dir, p))
                    .collect::<Vec<_>>()
                    .join(", ");
                let selected = overlay_dir_for_scope(engine, module_id, scope);
                warnings.push(format!(
                    "overlay layout ({scope_name}) module {module_id}: multiple overlay dirs exist: {found}; agentpack will use {} (consider migrating/removing legacy dirs)",
                    rel(&engine.repo.repo_dir, &selected)
                ));
            }

            for dir in existing {
                if !dir.is_dir() {
                    warnings.push(format!(
                        "overlay layout ({scope_name}) module {module_id}: {} exists but is not a directory",
                        rel(&engine.repo.repo_dir, &dir)
                    ));
                    continue;
                }

                let meta_path = dir.join(".agentpack").join("module_id");
                if !meta_path.exists() {
                    continue;
                }
                let raw = match std::fs::read_to_string(&meta_path) {
                    Ok(s) => s,
                    Err(err) => {
                        warnings.push(format!(
                            "overlay metadata ({scope_name}) module {module_id}: failed to read {}: {err}",
                            rel(&engine.repo.repo_dir, &meta_path)
                        ));
                        continue;
                    }
                };
                let got = raw.trim_end();
                if got != module_id {
                    warnings.push(format!(
                        "overlay metadata ({scope_name}) module {module_id}: {} contains {:?} (expected {:?})",
                        rel(&engine.repo.repo_dir, &meta_path),
                        got,
                        module_id
                    ));
                }
            }
        }
    }

    warnings
}
