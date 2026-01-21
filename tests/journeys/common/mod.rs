#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use assert_cmd::prelude::*;

pub const TEST_MACHINE_ID: &str = "test-machine";
pub const TEST_PROJECT_ORIGIN_URL: &str = "https://github.com/example/example.git";

pub fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(path, contents).expect("write file");
}

pub fn read_text_normalized(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .replace("\r\n", "\n")
}

pub fn run_out(env: &TestEnv, args: &[&str]) -> Output {
    env.agentpack().args(args).output().expect("run agentpack")
}

pub fn run_ok(env: &TestEnv, args: &[&str]) -> Output {
    let out = run_out(env, args);
    assert!(
        out.status.success(),
        "command failed: agentpack {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    out
}

pub fn run_fail(env: &TestEnv, args: &[&str]) -> Output {
    let out = run_out(env, args);
    assert!(
        !out.status.success(),
        "command unexpectedly succeeded: agentpack {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    out
}

pub fn parse_stdout_json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|err| {
        panic!(
            "parse json stdout: {err}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        )
    })
}

pub fn run_json_ok(env: &TestEnv, args: &[&str]) -> serde_json::Value {
    parse_stdout_json(&run_ok(env, args))
}

pub fn run_json_fail(env: &TestEnv, args: &[&str]) -> serde_json::Value {
    parse_stdout_json(&run_fail(env, args))
}

pub fn assert_error_code(envelope: &serde_json::Value, expected: &str) {
    assert_eq!(
        envelope["errors"][0]["code"].as_str(),
        Some(expected),
        "expected errors[0].code={expected}; got {}",
        envelope["errors"][0]
    );
}

pub fn git_out(dir: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git")
}

pub fn git_ok(dir: &Path, args: &[&str]) {
    let out = git_out(dir, args);
    assert!(
        out.status.success(),
        "git command failed: git {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

pub fn git_stdout(dir: &Path, args: &[&str]) -> String {
    let out = git_out(dir, args);
    assert!(
        out.status.success(),
        "git command failed: git {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8(out.stdout).expect("git stdout utf8")
}

pub fn git_clone_branch(remote: &Path, branch: &str, dst: &Path) {
    let remote = remote.to_string_lossy().to_string();
    let dst = dst.to_string_lossy().to_string();
    let out = Command::new("git")
        .args(["clone", "--branch", branch, &remote, &dst])
        .output()
        .expect("git clone");
    assert!(
        out.status.success(),
        "git clone failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

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
        write_file(
            self.manifest_path().as_path(),
            minimal_manifest_with_base_modules(),
        );
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
