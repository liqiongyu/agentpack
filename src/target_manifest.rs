use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::deploy::{ManagedPaths, TargetPath};
use crate::targets::TargetRoot;

pub const TARGET_MANIFEST_FILENAME: &str = ".agentpack.manifest.json";
const TARGET_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedManifestFile {
    pub path: String,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetManifest {
    pub schema_version: u32,
    pub generated_at: String,
    pub tool: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    pub managed_files: Vec<ManagedManifestFile>,
}

impl TargetManifest {
    pub fn new(tool: String, generated_at: String, snapshot_id: Option<String>) -> Self {
        Self {
            schema_version: TARGET_MANIFEST_SCHEMA_VERSION,
            generated_at,
            tool,
            snapshot_id,
            managed_files: Vec::new(),
        }
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let manifest: TargetManifest =
            serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
        if manifest.schema_version != TARGET_MANIFEST_SCHEMA_VERSION {
            anyhow::bail!(
                "unsupported target manifest schema_version: {}",
                manifest.schema_version
            );
        }
        Ok(manifest)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let mut out = serde_json::to_string_pretty(self).context("serialize target manifest")?;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        std::fs::write(path, out).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }
}

pub fn manifest_path(root: &Path) -> PathBuf {
    root.join(TARGET_MANIFEST_FILENAME)
}

pub fn load_managed_paths_from_manifests(roots: &[TargetRoot]) -> anyhow::Result<ManagedPaths> {
    let mut out = ManagedPaths::new();
    for root in roots {
        let path = manifest_path(&root.root);
        if !path.exists() {
            continue;
        }
        let manifest = TargetManifest::load(&path)?;
        for f in manifest.managed_files {
            ensure_safe_relative_path(&f.path)
                .with_context(|| format!("invalid manifest entry path: {}", f.path))?;
            out.insert(TargetPath {
                target: root.target.clone(),
                path: root.root.join(&f.path),
            });
        }
    }
    Ok(out)
}

fn ensure_safe_relative_path(p: &str) -> anyhow::Result<()> {
    let path = Path::new(p);
    if path.is_absolute() {
        anyhow::bail!("path must be relative: {p}");
    }
    for c in path.components() {
        if matches!(c, std::path::Component::ParentDir) {
            anyhow::bail!("path must not contain '..': {p}");
        }
    }
    Ok(())
}
