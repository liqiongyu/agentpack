use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::fs::write_atomic;
use crate::user_error::UserError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ModuleType {
    Instructions,
    Skill,
    Prompt,
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    #[serde(rename = "type")]
    pub module_type: ModuleType,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub targets: Vec<String>,
    pub source: Source,
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_yaml::Value>,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(default)]
    pub local_path: Option<LocalPathSource>,
    #[serde(default)]
    pub git: Option<GitSource>,
}

impl Source {
    pub fn kind(&self) -> SourceKind {
        match (&self.local_path, &self.git) {
            (Some(_), None) => SourceKind::LocalPath,
            (None, Some(_)) => SourceKind::Git,
            _ => SourceKind::Invalid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    LocalPath,
    Git,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalPathSource {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource {
    pub url: String,
    #[serde(rename = "ref", default = "default_git_ref")]
    pub ref_name: String,
    #[serde(default)]
    pub subdir: String,
    #[serde(default = "default_git_shallow")]
    pub shallow: bool,
}

fn default_git_ref() -> String {
    "main".to_string()
}

fn default_git_shallow() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub include_tags: Vec<String>,
    #[serde(default)]
    pub include_modules: Vec<String>,
    #[serde(default)]
    pub exclude_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetMode {
    Files,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetScope {
    User,
    Project,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub mode: TargetMode,
    pub scope: TargetScope,
    #[serde(default)]
    pub options: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    #[serde(default)]
    pub targets: BTreeMap<String, TargetConfig>,
    #[serde(default)]
    pub modules: Vec<Module>,
}

impl Manifest {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_CONFIG_MISSING",
                        format!("missing config manifest: {}", path.display()),
                    )
                    .with_details(serde_json::json!({
                        "path": path.to_string_lossy(),
                        "hint": "run `agentpack init` to create a repo skeleton",
                    })),
                ));
            }
            Err(err) => {
                return Err(err).with_context(|| format!("read {}", path.display()));
            }
        };

        let manifest: Manifest = serde_yaml::from_str(&raw).map_err(|err| {
            anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!("invalid config: {}", path.display()),
                )
                .with_details(serde_json::json!({
                    "path": path.to_string_lossy(),
                    "error": err.to_string(),
                })),
            )
        })?;
        validate_manifest(&manifest)?;
        Ok(manifest)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        validate_manifest(self)?;
        let mut out = serde_yaml::to_string(self).context("serialize manifest")?;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        write_atomic(path, out.as_bytes()).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }

    pub fn repo_root(&self, manifest_path: &Path) -> PathBuf {
        manifest_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

fn validate_manifest(manifest: &Manifest) -> anyhow::Result<()> {
    if manifest.version != 1 {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_UNSUPPORTED_VERSION",
                format!("unsupported manifest version: {}", manifest.version),
            )
            .with_details(serde_json::json!({ "version": manifest.version, "supported": [1] })),
        ));
    }

    for (target_name, cfg) in &manifest.targets {
        match target_name.as_str() {
            "codex" | "claude_code" => {}
            "cursor" => {
                if matches!(cfg.scope, TargetScope::User) {
                    return Err(anyhow::Error::new(
                        UserError::new(
                            "E_CONFIG_INVALID",
                            "cursor target does not support user scope",
                        )
                        .with_details(serde_json::json!({
                            "target": "cursor",
                            "allowed_scopes": ["project","both"],
                        })),
                    ));
                }
            }
            _ => {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_TARGET_UNSUPPORTED",
                        format!("unsupported target: {target_name}"),
                    )
                    .with_details(serde_json::json!({
                        "target": target_name,
                        "allowed": ["codex","claude_code","cursor"],
                    })),
                ));
            }
        }
    }

    if !manifest.profiles.contains_key("default") {
        return Err(anyhow::Error::new(
            UserError::new("E_CONFIG_INVALID", "missing required profile: default")
                .with_details(serde_json::json!({ "profile": "default" })),
        ));
    }

    let mut ids = BTreeSet::new();
    for m in &manifest.modules {
        if !ids.insert(m.id.clone()) {
            return Err(anyhow::Error::new(
                UserError::new("E_CONFIG_INVALID", format!("duplicate module id: {}", m.id))
                    .with_details(serde_json::json!({ "module_id": m.id })),
            ));
        }

        for t in &m.targets {
            if t != "codex" && t != "claude_code" && t != "cursor" {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_TARGET_UNSUPPORTED",
                        format!("module {} has unsupported target: {}", m.id, t),
                    )
                    .with_details(serde_json::json!({
                        "module_id": m.id,
                        "target": t,
                        "allowed": ["codex","claude_code","cursor"],
                    })),
                ));
            }
        }

        match m.source.kind() {
            SourceKind::LocalPath | SourceKind::Git => {}
            SourceKind::Invalid => {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_CONFIG_INVALID",
                        format!(
                            "module {} must have exactly one source type (local_path or git)",
                            m.id
                        ),
                    )
                    .with_details(serde_json::json!({ "module_id": m.id })),
                ));
            }
        }
    }

    Ok(())
}
