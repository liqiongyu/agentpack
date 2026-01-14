use std::path::{Path, PathBuf};
use std::process::Command;

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .env("AGENTPACK_MACHINE_ID", "test-machine")
        .env("HOME", home)
        .output()
        .expect("run agentpack")
}

fn git_in(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn init_workspace(tmp: &tempfile::TempDir) -> anyhow::Result<PathBuf> {
    let workspace = tmp.path().join("workspace");
    std::fs::create_dir_all(&workspace)?;

    assert!(git_in(&workspace, &["init"]).status.success());
    // Provide a stable origin for deterministic project_id derivation.
    let _ = git_in(
        &workspace,
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/example/example.git",
        ],
    );

    Ok(workspace)
}

fn write_manifest(repo_dir: &Path, codex_home: &Path) -> anyhow::Result<()> {
    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{codex_home}'
      write_agents_global: false
      write_agents_repo_root: false
      write_user_prompts: false
      write_user_skills: true
      write_repo_skills: false

modules:
  - id: skill:my-skill
    type: skill
    enabled: true
    tags: ["base"]
    targets: ["codex"]
    source:
      local_path:
        path: modules/skills/my-skill
"#,
        codex_home = codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest)?;
    Ok(())
}

fn write_skill(repo_dir: &Path, skill_md: &str) -> anyhow::Result<()> {
    let dir = repo_dir.join("modules/skills/my-skill");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("SKILL.md"), skill_md)?;
    Ok(())
}

#[test]
fn plan_json_succeeds_with_valid_skill_frontmatter() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(&tmp)?;

    let init = agentpack_in(home, &workspace, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    let codex_home = home.join("codex_home");
    std::fs::create_dir_all(&codex_home)?;

    write_manifest(&repo_dir, &codex_home)?;
    write_skill(
        &repo_dir,
        "---\nname: my-skill\ndescription: Example Skill\n---\n\n# my-skill\n",
    )?;

    let plan = agentpack_in(home, &workspace, &["--target", "codex", "plan", "--json"]);
    assert!(plan.status.success());

    let v = parse_stdout_json(&plan);
    assert_eq!(v["command"], "plan");
    assert!(v["ok"].as_bool().unwrap_or(false));

    Ok(())
}

#[test]
fn plan_json_fails_with_config_invalid_on_missing_skill_frontmatter() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(&tmp)?;

    let init = agentpack_in(home, &workspace, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    let codex_home = home.join("codex_home");
    std::fs::create_dir_all(&codex_home)?;

    write_manifest(&repo_dir, &codex_home)?;
    write_skill(&repo_dir, "# invalid skill (no frontmatter)\n")?;

    let plan = agentpack_in(home, &workspace, &["--target", "codex", "plan", "--json"]);
    assert!(!plan.status.success());

    let v = parse_stdout_json(&plan);
    assert_eq!(v["command"], "plan");
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_CONFIG_INVALID");

    Ok(())
}
