use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context as _;

use crate::config::TargetScope;
use crate::deploy::DesiredState;
use crate::fs::list_files;

pub(crate) fn insert_file(
    desired: &mut DesiredState,
    target: &str,
    path: PathBuf,
    bytes: Vec<u8>,
    module_ids: Vec<String>,
) -> anyhow::Result<()> {
    crate::deploy::insert_desired_file(desired, target, path, bytes, module_ids)
}

pub(crate) fn module_name_from_id(id: &str) -> Option<String> {
    id.split_once(':').map(|(_, name)| name.to_string())
}

pub(crate) fn first_file(dir: &Path) -> anyhow::Result<PathBuf> {
    let files = list_files(dir)?;
    files
        .into_iter()
        .min()
        .context("module directory contains no files")
}

pub(crate) fn expand_tilde(s: &str) -> anyhow::Result<PathBuf> {
    if let Some(rest) = s.strip_prefix("~/") {
        let home = dirs::home_dir().context("resolve home dir")?;
        return Ok(home.join(rest));
    }
    Ok(PathBuf::from(s))
}

pub(crate) fn scope_flags(scope: &TargetScope) -> (bool, bool) {
    match scope {
        TargetScope::User => (true, false),
        TargetScope::Project => (false, true),
        TargetScope::Both => (true, true),
    }
}

#[cfg(feature = "target-codex")]
pub(crate) fn codex_home_from_options(
    opts: &BTreeMap<String, serde_yaml::Value>,
) -> anyhow::Result<PathBuf> {
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

pub(crate) fn get_bool(
    map: &BTreeMap<String, serde_yaml::Value>,
    key: &str,
    default: bool,
) -> bool {
    match map.get(key) {
        Some(serde_yaml::Value::Bool(b)) => *b,
        Some(serde_yaml::Value::String(s)) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => true,
            "false" | "no" | "0" => false,
            _ => default,
        },
        _ => default,
    }
}
