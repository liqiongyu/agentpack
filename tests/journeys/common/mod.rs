#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use assert_cmd::prelude::*;

pub const TEST_MACHINE_ID: &str = "test-machine";
pub const TEST_PROJECT_ORIGIN_URL: &str = "https://github.com/example/example.git";

pub struct TestEnv {
    _tmp: tempfile::TempDir,
    home: PathBuf,
    agentpack_home: PathBuf,
    workspace: PathBuf,
}

impl TestEnv {
    pub fn new() -> Self {
        let tmp = tempfile::tempdir().expect("tempdir");

        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).expect("create home");

        let agentpack_home = tmp.path().join("agentpack_home");
        std::fs::create_dir_all(&agentpack_home).expect("create agentpack home");

        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&workspace).expect("create workspace");

        assert!(git_in(&workspace, &["init"]).status.success());
        // Provide a stable origin for deterministic project_id derivation.
        let add_origin = git_in(
            &workspace,
            &["remote", "add", "origin", TEST_PROJECT_ORIGIN_URL],
        );
        if !add_origin.status.success() {
            assert!(
                git_in(
                    &workspace,
                    &["remote", "set-url", "origin", TEST_PROJECT_ORIGIN_URL],
                )
                .status
                .success()
            );
        }

        // Ensure commits/branches work in upcoming journeys without global git config.
        let _ = git_in(&workspace, &["config", "user.email", "test@example.com"]);
        let _ = git_in(&workspace, &["config", "user.name", "Test User"]);

        Self {
            _tmp: tmp,
            home,
            agentpack_home,
            workspace,
        }
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn agentpack_home(&self) -> &Path {
        &self.agentpack_home
    }

    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    pub fn repo_dir(&self) -> PathBuf {
        self.agentpack_home.join("repo")
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.repo_dir().join("agentpack.yaml")
    }

    pub fn init_repo(&self) {
        self.agentpack()
            .args(["--json", "--yes", "init"])
            .assert()
            .success();
    }

    pub fn init_repo_with_base_modules(&self) {
        self.init_repo();
        std::fs::write(self.manifest_path(), minimal_manifest_with_base_modules())
            .expect("write agentpack.yaml");
    }

    pub fn agentpack(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_agentpack"));
        cmd.current_dir(&self.workspace)
            .env("AGENTPACK_HOME", &self.agentpack_home)
            .env("AGENTPACK_MACHINE_ID", TEST_MACHINE_ID)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("CODEX_HOME", self.home.join(".codex"))
            .env("XDG_CONFIG_HOME", &self.home)
            .env("XDG_CACHE_HOME", &self.home)
            .env("XDG_DATA_HOME", &self.home)
            .env("XDG_STATE_HOME", &self.home)
            // Prevent accidental editor popups in tests.
            .env("EDITOR", "");
        cmd
    }

    pub fn git(&self, args: &[&str]) -> Output {
        git_in(&self.workspace, args)
    }
}

fn git_in(dir: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git")
}

fn minimal_manifest_with_base_modules() -> &'static str {
    r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  codex:
    mode: files
    scope: both
    options:
      codex_home: ""
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

modules:
  - id: instructions:base
    type: instructions
    tags: ["base"]
    source:
      local_path:
        path: "modules/instructions/base"
  - id: prompt:draftpr
    type: prompt
    tags: ["base"]
    source:
      local_path:
        path: "modules/prompts/draftpr.md"
  - id: command:ap-plan
    type: command
    tags: ["base"]
    targets: ["claude_code"]
    source:
      local_path:
        path: "modules/claude-commands/ap-plan.md"
"#
}
