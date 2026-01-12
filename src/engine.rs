use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context as _;

use crate::config::{Manifest, Module, ModuleType, TargetScope};
use crate::deploy::DesiredState;
use crate::fs::list_files;
use crate::lockfile::Lockfile;
use crate::overlay::resolve_upstream_module_root;
use crate::paths::{AgentpackHome, RepoPaths};
use crate::project::ProjectContext;
use crate::store::{Store, sanitize_module_id};
use crate::target_adapters::adapter_for;
use crate::targets::{TargetRoot, dedup_roots};
use crate::user_error::UserError;
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

        let targets = self.targets_for_filter(target_filter)?;
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

    fn targets_for_filter(&self, filter: &str) -> anyhow::Result<Vec<String>> {
        let known: Vec<String> = self.manifest.targets.keys().cloned().collect();
        match filter {
            "all" => Ok(known),
            "codex" => Ok(vec!["codex".to_string()]),
            "claude_code" => Ok(vec!["claude_code".to_string()]),
            other => Err(anyhow::Error::new(
                UserError::new(
                    "E_TARGET_UNSUPPORTED",
                    format!("unsupported --target: {other}"),
                )
                .with_details(serde_json::json!({
                    "target": other,
                    "allowed": ["all","codex","claude_code"],
                })),
            )),
        }
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

    pub(crate) fn render_codex(
        &self,
        modules: &[&Module],
        desired: &mut DesiredState,
        warnings: &mut Vec<String>,
        roots: &mut Vec<TargetRoot>,
    ) -> anyhow::Result<()> {
        let target_cfg = self
            .manifest
            .targets
            .get("codex")
            .context("missing codex target config")?;
        let opts = &target_cfg.options;

        let codex_home = codex_home_from_options(opts)?;
        let (allow_user, allow_project) = scope_flags(&target_cfg.scope);
        let write_repo_skills = allow_project && get_bool(opts, "write_repo_skills", true);
        let write_user_skills = allow_user && get_bool(opts, "write_user_skills", true);
        let write_user_prompts = allow_user && get_bool(opts, "write_user_prompts", true);
        let write_agents_global = allow_user && get_bool(opts, "write_agents_global", true);
        let write_agents_repo_root =
            allow_project && get_bool(opts, "write_agents_repo_root", true);

        if write_agents_global {
            roots.push(TargetRoot {
                target: "codex".to_string(),
                root: codex_home.clone(),
                scan_extras: false,
            });
        }
        if write_user_prompts {
            roots.push(TargetRoot {
                target: "codex".to_string(),
                root: codex_home.join("prompts"),
                scan_extras: true,
            });
        }
        if write_user_skills {
            roots.push(TargetRoot {
                target: "codex".to_string(),
                root: codex_home.join("skills"),
                scan_extras: true,
            });
        }
        if write_agents_repo_root {
            roots.push(TargetRoot {
                target: "codex".to_string(),
                root: self.project.project_root.clone(),
                scan_extras: false,
            });
        }
        if write_repo_skills {
            roots.push(TargetRoot {
                target: "codex".to_string(),
                root: self.project.project_root.join(".codex/skills"),
                scan_extras: true,
            });
        }

        let mut instructions_parts: Vec<(String, String)> = Vec::new();
        for m in modules
            .iter()
            .filter(|m| matches!(m.module_type, ModuleType::Instructions))
            .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "codex"))
        {
            let (_tmp, materialized) = self.materialize_module(m, warnings)?;
            let agents_path = materialized.join("AGENTS.md");
            if agents_path.exists() {
                instructions_parts.push((
                    m.id.clone(),
                    std::fs::read_to_string(&agents_path)
                        .with_context(|| format!("read {}", agents_path.display()))?,
                ));
            }
        }

        if !instructions_parts.is_empty() {
            let module_ids: Vec<String> = instructions_parts
                .iter()
                .map(|(id, _)| id.clone())
                .collect();
            let combined = instructions_parts
                .into_iter()
                .map(|(_, text)| text)
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");
            let bytes = combined.into_bytes();

            if write_agents_global {
                insert_file(
                    desired,
                    "codex",
                    codex_home.join("AGENTS.md"),
                    bytes.clone(),
                    module_ids.clone(),
                )?;
            }
            if write_agents_repo_root {
                insert_file(
                    desired,
                    "codex",
                    self.project.project_root.join("AGENTS.md"),
                    bytes,
                    module_ids,
                )?;
            }
        }

        for m in modules
            .iter()
            .filter(|m| matches!(m.module_type, ModuleType::Prompt))
            .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "codex"))
        {
            if !write_user_prompts {
                continue;
            }
            let (_tmp, materialized) = self.materialize_module(m, warnings)?;
            let prompt_file = first_file(&materialized)?;
            let name = prompt_file
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("prompt.md");
            let bytes = std::fs::read(&prompt_file)?;
            insert_file(
                desired,
                "codex",
                codex_home.join("prompts").join(name),
                bytes,
                vec![m.id.clone()],
            )?;
        }

        for m in modules
            .iter()
            .filter(|m| matches!(m.module_type, ModuleType::Skill))
            .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "codex"))
        {
            let (_tmp, materialized) = self.materialize_module(m, warnings)?;
            let skill_name =
                module_name_from_id(&m.id).unwrap_or_else(|| sanitize_module_id(&m.id));

            let files = list_files(&materialized)?;
            for f in files {
                let rel = f
                    .strip_prefix(&materialized)
                    .unwrap_or(&f)
                    .to_string_lossy()
                    .replace('\\', "/");
                let bytes = std::fs::read(&f)?;

                if write_user_skills {
                    let dst = codex_home.join("skills").join(&skill_name).join(&rel);
                    insert_file(desired, "codex", dst, bytes.clone(), vec![m.id.clone()])?;
                }
                if write_repo_skills {
                    let dst = self
                        .project
                        .project_root
                        .join(".codex/skills")
                        .join(&skill_name)
                        .join(&rel);
                    insert_file(desired, "codex", dst, bytes, vec![m.id.clone()])?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn render_claude_code(
        &self,
        modules: &[&Module],
        desired: &mut DesiredState,
        warnings: &mut Vec<String>,
        roots: &mut Vec<TargetRoot>,
    ) -> anyhow::Result<()> {
        let target_cfg = self
            .manifest
            .targets
            .get("claude_code")
            .context("missing claude_code target config")?;
        let opts = &target_cfg.options;

        let (allow_user, allow_project) = scope_flags(&target_cfg.scope);
        let write_repo_commands = allow_project && get_bool(opts, "write_repo_commands", true);
        let write_user_commands = allow_user && get_bool(opts, "write_user_commands", true);

        let user_commands_dir = expand_tilde("~/.claude/commands")?;

        if write_user_commands {
            roots.push(TargetRoot {
                target: "claude_code".to_string(),
                root: user_commands_dir.clone(),
                scan_extras: true,
            });
        }
        if write_repo_commands {
            roots.push(TargetRoot {
                target: "claude_code".to_string(),
                root: self.project.project_root.join(".claude/commands"),
                scan_extras: true,
            });
        }

        for m in modules
            .iter()
            .filter(|m| matches!(m.module_type, ModuleType::Command))
            .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "claude_code"))
        {
            let (_tmp, materialized) = self.materialize_module(m, warnings)?;
            let cmd_file = first_file(&materialized)?;
            let name = cmd_file
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("command.md");
            let bytes = std::fs::read(&cmd_file)?;

            if write_user_commands {
                insert_file(
                    desired,
                    "claude_code",
                    user_commands_dir.join(name),
                    bytes.clone(),
                    vec![m.id.clone()],
                )?;
            }
            if write_repo_commands {
                insert_file(
                    desired,
                    "claude_code",
                    self.project
                        .project_root
                        .join(".claude/commands")
                        .join(name),
                    bytes,
                    vec![m.id.clone()],
                )?;
            }
        }

        Ok(())
    }

    fn materialize_module(
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

        let overlays: [&Path; 3] = [&global, &machine, &project];
        crate::overlay::compose_module_tree(&upstream, &overlays, &dst)?;
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

fn insert_file(
    desired: &mut DesiredState,
    target: &str,
    path: PathBuf,
    bytes: Vec<u8>,
    module_ids: Vec<String>,
) -> anyhow::Result<()> {
    crate::deploy::insert_desired_file(desired, target, path, bytes, module_ids)
}

fn module_name_from_id(id: &str) -> Option<String> {
    id.split_once(':').map(|(_, name)| name.to_string())
}

fn first_file(dir: &Path) -> anyhow::Result<PathBuf> {
    let files = list_files(dir)?;
    files
        .into_iter()
        .min()
        .context("module directory contains no files")
}

fn expand_tilde(s: &str) -> anyhow::Result<PathBuf> {
    if let Some(rest) = s.strip_prefix("~/") {
        let home = dirs::home_dir().context("resolve home dir")?;
        return Ok(home.join(rest));
    }
    Ok(PathBuf::from(s))
}

fn scope_flags(scope: &TargetScope) -> (bool, bool) {
    match scope {
        TargetScope::User => (true, false),
        TargetScope::Project => (false, true),
        TargetScope::Both => (true, true),
    }
}

fn codex_home_from_options(opts: &BTreeMap<String, serde_yaml::Value>) -> anyhow::Result<PathBuf> {
    if let Some(serde_yaml::Value::String(s)) = opts.get("codex_home") {
        if !s.trim().is_empty() {
            return expand_tilde(s);
        }
    }

    if let Ok(env) = std::env::var("CODEX_HOME") {
        if !env.trim().is_empty() {
            return expand_tilde(&env);
        }
    }

    expand_tilde("~/.codex")
}

fn get_bool(map: &BTreeMap<String, serde_yaml::Value>, key: &str, default: bool) -> bool {
    match map.get(key) {
        Some(serde_yaml::Value::Bool(b)) => *b,
        Some(serde_yaml::Value::String(s)) => s == "true" || s == "1",
        _ => default,
    }
}
