use std::path::{Path, PathBuf};

use anyhow::Context as _;

#[derive(Debug, Clone)]
pub struct AgentpackHome {
    pub root: PathBuf,
    pub repo_dir: PathBuf,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub snapshots_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AgentpackHome {
    pub fn resolve() -> anyhow::Result<Self> {
        let root = if let Ok(val) = std::env::var("AGENTPACK_HOME") {
            expand_tilde(&val)?
        } else {
            dirs::home_dir()
                .context("failed to resolve home directory")?
                .join(".agentpack")
        };

        let state_dir = root.join("state");
        Ok(Self {
            repo_dir: root.join("repo"),
            cache_dir: root.join("cache"),
            state_dir: state_dir.clone(),
            snapshots_dir: state_dir.join("snapshots"),
            logs_dir: state_dir.join("logs"),
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
        let base_instructions_dir = self.repo_dir.join("modules/instructions/base");
        let prompts_dir = self.repo_dir.join("modules/prompts");
        let claude_commands_dir = self.repo_dir.join("modules/claude-commands");

        std::fs::create_dir_all(&base_instructions_dir).ok();
        std::fs::create_dir_all(&prompts_dir).ok();
        std::fs::create_dir_all(&claude_commands_dir).ok();

        let base_agents = base_instructions_dir.join("AGENTS.md");
        if !base_agents.exists() {
            std::fs::write(&base_agents, default_base_agents_md())
                .context("write base AGENTS.md")?;
        }

        let draft_prompt = prompts_dir.join("draftpr.md");
        if !draft_prompt.exists() {
            std::fs::write(&draft_prompt, default_draft_prompt_md())
                .context("write draft prompt")?;
        }

        let ap_plan = claude_commands_dir.join("ap-plan.md");
        if !ap_plan.exists() {
            std::fs::write(&ap_plan, default_ap_plan_md()).context("write ap-plan command")?;
        }
        if !self.manifest_path.exists() {
            std::fs::write(&self.manifest_path, default_manifest()).context("write manifest")?;
        }
        Ok(())
    }
}

fn expand_tilde(s: &str) -> anyhow::Result<PathBuf> {
    if let Some(rest) = s.strip_prefix("~/") {
        let home = dirs::home_dir().context("resolve home dir")?;
        return Ok(home.join(rest));
    }
    Ok(PathBuf::from(s))
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

fn default_base_agents_md() -> &'static str {
    r#"# Agent Instructions (base)

This is a minimal `AGENTS.md` template managed by agentpack.

- Follow repository conventions and run tests before making changes.
- Prefer small, reviewable commits and keep diffs focused.
"#
}

fn default_draft_prompt_md() -> &'static str {
    r#"# draftpr

Draft a pull request description:
- Summary
- Motivation
- Changes
- Testing
- Rollout / Risk
"#
}

fn default_ap_plan_md() -> &'static str {
    r#"---
description: "agentpack: show plan"
---

Run `agentpack plan --json` and summarize the changes.
"#
}
