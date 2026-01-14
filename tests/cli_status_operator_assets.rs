use std::path::{Path, PathBuf};
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

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn write_manifest(repo_dir: &Path, codex_home: &Path) {
    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: []

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{codex_home}'
  claude_code:
    mode: files
    scope: project
    options: {{}}

modules: []
"#,
        codex_home = codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");
}

fn write_manifest_with_claude_repo_skills(repo_dir: &Path, codex_home: &Path) {
    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: []

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{codex_home}'
  claude_code:
    mode: files
    scope: project
    options:
      write_repo_commands: true
      write_user_commands: false
      write_repo_skills: true
      write_user_skills: false

modules: []
"#,
        codex_home = codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");
}

fn write_claude_operator_commands(dir: &Path, version: &str) {
    std::fs::create_dir_all(dir).expect("create commands dir");
    for name in [
        "ap-doctor.md",
        "ap-update.md",
        "ap-preview.md",
        "ap-plan.md",
        "ap-deploy.md",
        "ap-status.md",
        "ap-diff.md",
        "ap-explain.md",
        "ap-evolve.md",
    ] {
        std::fs::write(
            dir.join(name),
            format!("---\nagentpack_version: \"{version}\"\n---\n\nHello\n"),
        )
        .expect("write operator command");
    }
}

fn init_workspace(tmp: &tempfile::TempDir) -> (PathBuf, PathBuf) {
    let home = tmp.path().to_path_buf();
    let workspace = tmp.path().join("workspace");
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

    (home, workspace)
}

#[test]
fn status_warns_when_codex_operator_assets_are_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (home, workspace) = init_workspace(&tmp);
    let repo_dir = home.join("repo");

    let codex_home = workspace.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");
    write_manifest(&repo_dir, &codex_home);

    let status = agentpack_in(
        &home,
        &workspace,
        &["--target", "codex", "status", "--json"],
    );
    assert!(status.status.success());

    let json = parse_stdout_json(&status);
    let warnings = json["warnings"].as_array().expect("warnings array");
    assert!(warnings.iter().any(|w| {
        w.as_str()
            .unwrap_or_default()
            .contains("operator assets missing (codex/user)")
    }));
}

#[test]
fn status_warns_when_codex_operator_assets_are_outdated() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (home, workspace) = init_workspace(&tmp);
    let repo_dir = home.join("repo");

    let codex_home = workspace.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");
    write_manifest(&repo_dir, &codex_home);

    let operator = codex_home.join("skills/agentpack-operator/SKILL.md");
    std::fs::create_dir_all(operator.parent().expect("parent")).expect("create operator dir");
    std::fs::write(
        &operator,
        "<!-- agentpack_version: 0.0.0 -->\n\n# agentpack-operator\n",
    )
    .expect("write operator skill");

    let status = agentpack_in(
        &home,
        &workspace,
        &["--target", "codex", "status", "--json"],
    );
    assert!(status.status.success());

    let json = parse_stdout_json(&status);
    let warnings = json["warnings"].as_array().expect("warnings array");
    assert!(warnings.iter().any(|w| {
        w.as_str()
            .unwrap_or_default()
            .contains("operator assets outdated (codex/user)")
    }));
}

#[test]
fn status_warns_when_claude_operator_assets_are_outdated() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (home, workspace) = init_workspace(&tmp);
    let repo_dir = home.join("repo");

    let codex_home = workspace.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");
    write_manifest(&repo_dir, &codex_home);

    let dir = workspace.join(".claude/commands");
    std::fs::create_dir_all(&dir).expect("create commands dir");
    std::fs::write(
        dir.join("ap-plan.md"),
        "---\nagentpack_version: \"0.0.0\"\n---\n\nHello\n",
    )
    .expect("write ap-plan");

    let status = agentpack_in(
        &home,
        &workspace,
        &["--target", "claude_code", "status", "--json"],
    );
    assert!(status.status.success());

    let json = parse_stdout_json(&status);
    let warnings = json["warnings"].as_array().expect("warnings array");
    assert!(warnings.iter().any(|w| {
        w.as_str()
            .unwrap_or_default()
            .contains("operator assets outdated (claude_code/project)")
    }));
}

#[test]
fn status_warns_when_claude_operator_skill_is_missing_when_enabled() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (home, workspace) = init_workspace(&tmp);
    let repo_dir = home.join("repo");

    let codex_home = workspace.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");
    write_manifest_with_claude_repo_skills(&repo_dir, &codex_home);

    write_claude_operator_commands(
        &workspace.join(".claude/commands"),
        env!("CARGO_PKG_VERSION"),
    );

    let status = agentpack_in(
        &home,
        &workspace,
        &["--target", "claude_code", "status", "--json"],
    );
    assert!(status.status.success());

    let json = parse_stdout_json(&status);
    let warnings = json["warnings"].as_array().expect("warnings array");
    assert!(warnings.iter().any(|w| {
        let w = w.as_str().unwrap_or_default();
        w.contains("operator assets missing (claude_code/project)")
            && w.contains(".claude/skills/agentpack-operator/SKILL.md")
    }));
}

#[test]
fn status_warns_when_claude_operator_skill_is_outdated_when_enabled() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (home, workspace) = init_workspace(&tmp);
    let repo_dir = home.join("repo");

    let codex_home = workspace.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");
    write_manifest_with_claude_repo_skills(&repo_dir, &codex_home);

    write_claude_operator_commands(
        &workspace.join(".claude/commands"),
        env!("CARGO_PKG_VERSION"),
    );

    let operator = workspace.join(".claude/skills/agentpack-operator/SKILL.md");
    std::fs::create_dir_all(operator.parent().expect("parent")).expect("create operator dir");
    std::fs::write(
        &operator,
        "---\nagentpack_version: \"0.0.0\"\n---\n\n# agentpack-operator\n",
    )
    .expect("write operator skill");

    let status = agentpack_in(
        &home,
        &workspace,
        &["--target", "claude_code", "status", "--json"],
    );
    assert!(status.status.success());

    let json = parse_stdout_json(&status);
    let warnings = json["warnings"].as_array().expect("warnings array");
    assert!(warnings.iter().any(|w| {
        let w = w.as_str().unwrap_or_default();
        w.contains("operator assets outdated (claude_code/project)")
            && w.contains(".claude/skills/agentpack-operator/SKILL.md")
    }));
}
