use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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
        let raw =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let manifest: Manifest =
            serde_yaml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
        validate_manifest(&manifest)?;
        Ok(manifest)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        validate_manifest(self)?;
        let mut out = serde_yaml::to_string(self).context("serialize manifest")?;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        std::fs::write(path, out).with_context(|| format!("write {}", path.display()))?;
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
        anyhow::bail!("unsupported manifest version: {}", manifest.version);
    }

    for target_name in manifest.targets.keys() {
        if target_name != "codex" && target_name != "claude_code" {
            anyhow::bail!("unsupported target: {target_name}");
        }
    }

    if !manifest.profiles.contains_key("default") {
        anyhow::bail!("missing required profile: default");
    }

    let mut ids = BTreeSet::new();
    for m in &manifest.modules {
        if !ids.insert(m.id.clone()) {
            anyhow::bail!("duplicate module id: {}", m.id);
        }

        for t in &m.targets {
            if t != "codex" && t != "claude_code" {
                anyhow::bail!("module {} has unsupported target: {}", m.id, t);
            }
        }

        match m.source.kind() {
            SourceKind::LocalPath | SourceKind::Git => {}
            SourceKind::Invalid => {
                anyhow::bail!(
                    "module {} must have exactly one source type (local_path or git)",
                    m.id
                );
            }
        }
    }

    Ok(())
}
