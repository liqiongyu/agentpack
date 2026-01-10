use std::path::{Path, PathBuf};

use anyhow::Context as _;

#[derive(Debug, Clone)]
pub struct AgentpackHome {
    pub root: PathBuf,
    pub repo_dir: PathBuf,
    pub store_dir: PathBuf,
    pub state_dir: PathBuf,
    pub deployments_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AgentpackHome {
    pub fn resolve() -> anyhow::Result<Self> {
        let root = if let Ok(val) = std::env::var("AGENTPACK_HOME") {
            PathBuf::from(val)
        } else {
            dirs::data_local_dir()
                .context("failed to resolve OS data directory")?
                .join("agentpack")
        };

        Ok(Self {
            repo_dir: root.join("repo"),
            store_dir: root.join("store"),
            state_dir: root.join("state"),
            deployments_dir: root.join("state").join("deployments"),
            logs_dir: root.join("logs"),
            root,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RepoPaths {
    pub repo_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub lockfile_path: PathBuf,
}

impl RepoPaths {
    pub fn resolve(home: &AgentpackHome, repo_override: Option<&Path>) -> anyhow::Result<Self> {
        let repo_dir = repo_override
            .map(PathBuf::from)
            .unwrap_or_else(|| home.repo_dir.clone());

        Ok(Self {
            manifest_path: repo_dir.join("agentpack.yaml"),
            lockfile_path: repo_dir.join("agentpack.lock.json"),
            repo_dir,
        })
    }

    pub fn init_repo_skeleton(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.repo_dir).context("create repo dir")?;
        std::fs::create_dir_all(self.repo_dir.join("modules/instructions")).ok();
        std::fs::create_dir_all(self.repo_dir.join("modules/prompts")).ok();
        std::fs::create_dir_all(self.repo_dir.join("modules/claude-commands")).ok();
        if !self.manifest_path.exists() {
            std::fs::write(&self.manifest_path, default_manifest()).context("write manifest")?;
        }
        Ok(())
    }
}

fn default_manifest() -> &'static str {
    r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  codex:
    mode: files
    scope: both
    options:
      codex_home: "~/.codex"
      write_repo_skills: true
      write_user_skills: true
      write_user_prompts: true
      write_agents_global: true
      write_agents_repo_root: true
  claude_code:
    mode: files
    scope: both
    options:
      write_repo_commands: true
      write_user_commands: true
      write_repo_skills: false
      write_user_skills: false

modules: []
"#
}
