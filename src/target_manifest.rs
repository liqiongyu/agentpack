use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::deploy::{ManagedPaths, TargetPath};
use crate::fs::write_atomic;
use crate::targets::TargetRoot;

pub const LEGACY_TARGET_MANIFEST_FILENAME: &str = ".agentpack.manifest.json";
pub const TARGET_MANIFEST_GITIGNORE_LINE: &str = ".agentpack.manifest*.json";
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
        write_atomic(path, out.as_bytes()).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }
}

pub fn manifest_path_for_target(root: &Path, target: &str) -> PathBuf {
    root.join(manifest_filename(target))
}

pub fn legacy_manifest_path(root: &Path) -> PathBuf {
    root.join(LEGACY_TARGET_MANIFEST_FILENAME)
}

pub fn manifest_filename(target: &str) -> String {
    let safe = crate::store::sanitize_module_id(target);
    format!(".agentpack.manifest.{safe}.json")
}

pub fn is_target_manifest_filename(name: &str) -> bool {
    name == LEGACY_TARGET_MANIFEST_FILENAME
        || (name.starts_with(".agentpack.manifest.") && name.ends_with(".json"))
}

pub fn is_target_manifest_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .is_some_and(is_target_manifest_filename)
}

pub struct ManagedPathsFromManifests {
    pub managed_paths: ManagedPaths,
    pub warnings: Vec<String>,
}

pub(crate) fn read_target_manifest_soft(
    path: &Path,
    expected_target: &str,
) -> (Option<TargetManifest>, Vec<String>) {
    let mut warnings = Vec::new();

    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            warnings.push(format!(
                "target manifest ({}): failed to read {} (treating as missing): {err}",
                expected_target,
                path.display()
            ));
            return (None, warnings);
        }
    };

    let v: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(err) => {
            warnings.push(format!(
                "target manifest ({}): failed to parse {} (treating as missing): {err}",
                expected_target,
                path.display()
            ));
            return (None, warnings);
        }
    };

    let schema_version = v
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32;
    if schema_version != TARGET_MANIFEST_SCHEMA_VERSION {
        warnings.push(format!(
            "target manifest ({}): unsupported schema_version {} in {} (expected {}; treating as missing)",
            expected_target,
            schema_version,
            path.display(),
            TARGET_MANIFEST_SCHEMA_VERSION,
        ));
        return (None, warnings);
    }

    match serde_json::from_value::<TargetManifest>(v) {
        Ok(m) => {
            if m.tool != expected_target {
                warnings.push(format!(
                    "target manifest ({}): ignored {} because manifest.tool={} (expected {}; treating as missing)",
                    expected_target,
                    path.display(),
                    m.tool,
                    expected_target,
                ));
                return (None, warnings);
            }
            (Some(m), warnings)
        }
        Err(err) => {
            warnings.push(format!(
                "target manifest ({}): failed to parse {} (treating as missing): {err}",
                expected_target,
                path.display()
            ));
            (None, warnings)
        }
    }
}

pub fn load_managed_paths_from_manifests(
    roots: &[TargetRoot],
) -> anyhow::Result<ManagedPathsFromManifests> {
    let mut out = ManagedPaths::new();
    let mut warnings: Vec<String> = Vec::new();
    for root in roots {
        let preferred = manifest_path_for_target(&root.root, &root.target);
        let legacy = legacy_manifest_path(&root.root);

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

        let (manifest, manifest_warnings) = read_target_manifest_soft(&path, &root.target);
        warnings.extend(manifest_warnings);
        let Some(manifest) = manifest else {
            continue;
        };
        for f in manifest.managed_files {
            if let Err(err) = ensure_safe_relative_path(&f.path) {
                warnings.push(format!(
                    "target manifest ({}): skipped invalid entry path {:?} in {}: {err}",
                    root.target,
                    f.path,
                    path.display()
                ));
                continue;
            }
            out.insert(TargetPath {
                target: root.target.clone(),
                path: root.root.join(&f.path),
            });
        }
    }
    Ok(ManagedPathsFromManifests {
        managed_paths: out,
        warnings,
    })
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
