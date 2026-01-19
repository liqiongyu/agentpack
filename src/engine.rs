use std::path::{Path, PathBuf};

use anyhow::Context as _;

use crate::config::{Manifest, Module};
use crate::deploy::DesiredState;
use crate::lockfile::Lockfile;
use crate::overlay::resolve_upstream_module_root;
use crate::paths::{AgentpackHome, RepoPaths};
use crate::project::ProjectContext;
use crate::store::{Store, sanitize_module_id};
use crate::target_adapters::adapter_for;
use crate::targets::{TargetRoot, dedup_roots};
use crate::validate::validate_materialized_module;

#[derive(Debug)]
pub struct Engine {
    pub home: AgentpackHome,
    pub repo: RepoPaths,
    pub manifest: Manifest,
    pub lockfile: Option<Lockfile>,
    pub store: Store,
    pub project: ProjectContext,
    pub machine_id: String,
}

#[derive(Debug)]
pub struct RenderResult {
    pub desired: DesiredState,
    pub warnings: Vec<String>,
    pub roots: Vec<TargetRoot>,
}

impl Engine {
    pub fn load(
        repo_override: Option<&Path>,
        machine_override: Option<&str>,
    ) -> anyhow::Result<Self> {
        let home = AgentpackHome::resolve()?;
        let repo = RepoPaths::resolve(&home, repo_override)?;
        let manifest = Manifest::load(&repo.manifest_path).context("load manifest")?;
        let lockfile = Lockfile::load(&repo.lockfile_path).ok();
        let store = Store::new(&home);
        let cwd = std::env::current_dir().context("get cwd")?;
        let project = ProjectContext::detect(&cwd).context("detect project")?;
        let machine_id = if let Some(m) = machine_override {
            let normalized = crate::machine::normalize_machine_id(m);
            if normalized.is_empty() {
                crate::machine::detect_machine_id()?
            } else {
                normalized
            }
        } else {
            crate::machine::detect_machine_id()?
        };
        Ok(Self {
            home,
            repo,
            manifest,
            lockfile,
            store,
            project,
            machine_id,
        })
    }

    pub fn desired_state(
        &self,
        profile: &str,
        target_filter: &str,
    ) -> anyhow::Result<RenderResult> {
        let modules = self.select_modules(profile)?;
        let mut desired = DesiredState::new();
        let mut warnings = Vec::new();
        let mut roots = Vec::new();

        let targets = crate::target_selection::selected_targets(&self.manifest, target_filter)?;
        for target in targets {
            if let Some(adapter) = adapter_for(target.as_str()) {
                adapter.render(self, &modules, &mut desired, &mut warnings, &mut roots)?;
            }
        }

        Ok(RenderResult {
            desired,
            warnings,
            roots: dedup_roots(roots),
        })
    }

    fn select_modules(&self, profile_name: &str) -> anyhow::Result<Vec<&Module>> {
        let profile = self
            .manifest
            .profiles
            .get(profile_name)
            .with_context(|| format!("profile not found: {profile_name}"))?;

        let include_tags: std::collections::BTreeSet<_> = profile.include_tags.iter().collect();
        let include_ids: std::collections::BTreeSet<_> = profile.include_modules.iter().collect();
        let exclude_ids: std::collections::BTreeSet<_> = profile.exclude_modules.iter().collect();

        let mut out = Vec::new();
        for m in &self.manifest.modules {
            if !m.enabled || exclude_ids.contains(&m.id) {
                continue;
            }
            let tag_match = m.tags.iter().any(|t| include_tags.contains(t));
            let id_match = include_ids.contains(&m.id);
            if tag_match || id_match {
                out.push(m);
            }
        }

        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    pub(crate) fn materialize_module(
        &self,
        module: &Module,
        warnings: &mut Vec<String>,
    ) -> anyhow::Result<(tempfile::TempDir, PathBuf)> {
        let tmp = tempfile::tempdir().context("create tempdir")?;
        let dst = tmp.path().join(sanitize_module_id(&module.id));
        std::fs::create_dir_all(&dst).context("create module dir")?;

        let upstream = resolve_upstream_module_root(&self.home, &self.repo, module)?;
        let global = overlay_dir_global(&self.repo.repo_dir, &module.id);
        let machine = overlay_dir_machine(&self.repo.repo_dir, &self.machine_id, &module.id);
        let project =
            overlay_dir_project(&self.repo.repo_dir, &self.project.project_id, &module.id);

        let global = overlay_dir_prefer_existing(
            &global,
            &overlay_dir_global_fallbacks(&self.repo.repo_dir, &module.id),
        );
        let machine = overlay_dir_prefer_existing(
            &machine,
            &overlay_dir_machine_fallbacks(&self.repo.repo_dir, &self.machine_id, &module.id),
        );
        let project = overlay_dir_prefer_existing(
            &project,
            &overlay_dir_project_fallbacks(
                &self.repo.repo_dir,
                &self.project.project_id,
                &module.id,
            ),
        );

        warnings.extend(crate::overlay::overlay_drift_warnings(
            &module.id, "global", &upstream, &global,
        )?);
        warnings.extend(crate::overlay::overlay_drift_warnings(
            &module.id, "machine", &upstream, &machine,
        )?);
        warnings.extend(crate::overlay::overlay_drift_warnings(
            &module.id, "project", &upstream, &project,
        )?);

        let overlays = [
            crate::overlay::OverlayLayer {
                scope: "global",
                dir: &global,
            },
            crate::overlay::OverlayLayer {
                scope: "machine",
                dir: &machine,
            },
            crate::overlay::OverlayLayer {
                scope: "project",
                dir: &project,
            },
        ];
        crate::overlay::compose_module_tree(&module.id, &upstream, &overlays, &dst)?;
        validate_materialized_module(&module.module_type, &module.id, &dst)
            .context("validate module")?;

        Ok((tmp, dst))
    }
}

fn overlay_dir_global(repo_dir: &Path, module_id: &str) -> PathBuf {
    repo_dir
        .join("overlays")
        .join(crate::ids::module_fs_key(module_id))
}

fn overlay_dir_global_fallbacks(repo_dir: &Path, module_id: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();

    let bounded = crate::ids::module_fs_key(module_id);
    let unbounded = crate::ids::module_fs_key_unbounded(module_id);
    if bounded != unbounded {
        out.push(repo_dir.join("overlays").join(unbounded));
    }

    if crate::ids::is_safe_legacy_path_component(module_id) {
        out.push(repo_dir.join("overlays").join(module_id));
    }

    out
}

fn overlay_dir_machine(repo_dir: &Path, machine_id: &str, module_id: &str) -> PathBuf {
    repo_dir
        .join("overlays/machines")
        .join(machine_id)
        .join(crate::ids::module_fs_key(module_id))
}

fn overlay_dir_machine_fallbacks(
    repo_dir: &Path,
    machine_id: &str,
    module_id: &str,
) -> Vec<PathBuf> {
    let mut out = Vec::new();

    let bounded = crate::ids::module_fs_key(module_id);
    let unbounded = crate::ids::module_fs_key_unbounded(module_id);
    if bounded != unbounded {
        out.push(
            repo_dir
                .join("overlays/machines")
                .join(machine_id)
                .join(unbounded),
        );
    }

    if crate::ids::is_safe_legacy_path_component(module_id) {
        out.push(
            repo_dir
                .join("overlays/machines")
                .join(machine_id)
                .join(module_id),
        );
    }

    out
}

fn overlay_dir_project(repo_dir: &Path, project_id: &str, module_id: &str) -> PathBuf {
    repo_dir
        .join("projects")
        .join(project_id)
        .join("overlays")
        .join(crate::ids::module_fs_key(module_id))
}

fn overlay_dir_project_fallbacks(
    repo_dir: &Path,
    project_id: &str,
    module_id: &str,
) -> Vec<PathBuf> {
    let mut out = Vec::new();

    let bounded = crate::ids::module_fs_key(module_id);
    let unbounded = crate::ids::module_fs_key_unbounded(module_id);
    if bounded != unbounded {
        out.push(
            repo_dir
                .join("projects")
                .join(project_id)
                .join("overlays")
                .join(unbounded),
        );
    }

    if crate::ids::is_safe_legacy_path_component(module_id) {
        out.push(
            repo_dir
                .join("projects")
                .join(project_id)
                .join("overlays")
                .join(module_id),
        );
    }

    out
}

fn overlay_dir_prefer_existing(canonical: &Path, fallbacks: &[PathBuf]) -> PathBuf {
    if canonical.exists() {
        return canonical.to_path_buf();
    }

    for fallback in fallbacks {
        if fallback.exists() {
            return fallback.to_path_buf();
        }
    }

    canonical.to_path_buf()
}
