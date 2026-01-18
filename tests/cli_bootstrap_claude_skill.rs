use std::path::Path;
use std::process::Command;

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .env("HOME", home)
        .output()
        .expect("run agentpack")
}

fn write_manifest(repo_dir: &Path) {
    let manifest = r#"version: 1

profiles:
  default:
    include_tags: []

targets:
  claude_code:
    mode: files
    scope: project
    options:
      write_repo_commands: true
      write_user_commands: false
      write_repo_skills: true
      write_user_skills: false

modules: []
"#;
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");
}

#[test]
fn bootstrap_installs_claude_operator_skill_when_enabled() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path().to_path_buf();

    let workspace = home.join("workspace");
    std::fs::create_dir_all(&workspace).expect("create workspace");

    assert!(
        Command::new("git")
            .current_dir(&workspace)
            .args(["init"])
            .output()
            .expect("git init")
            .status
            .success()
    );

    let init = agentpack_in(&home, &workspace, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    write_manifest(&repo_dir);

    let out = agentpack_in(
        &home,
        &workspace,
        &[
            "--target",
            "claude_code",
            "bootstrap",
            "--scope",
            "project",
            "--yes",
            "--json",
        ],
    );
    assert!(out.status.success());

    assert!(
        workspace
            .join(".claude/skills/agentpack-operator/SKILL.md")
            .exists()
    );
    assert!(
        workspace
            .join(".claude/skills/.agentpack.manifest.claude_code.json")
            .exists()
    );
}
