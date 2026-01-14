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

#[test]
fn deploy_writes_claude_code_skill_modules_when_enabled() {
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
    let skill_dir = repo_dir.join("modules/skills/my-skill");
    std::fs::create_dir_all(&skill_dir).expect("create skill dir");
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: my-skill\ndescription: Example Skill for tests\n---\n\n# my-skill\n",
    )
    .expect("write SKILL.md");

    std::fs::write(
        repo_dir.join("agentpack.yaml"),
        r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  claude_code:
    mode: files
    scope: project
    options:
      write_repo_commands: false
      write_user_commands: false
      write_repo_skills: true
      write_user_skills: false

modules:
  - id: skill:my-skill
    type: skill
    tags: ["base"]
    targets: ["claude_code"]
    source:
      local_path:
        path: "modules/skills/my-skill"
"#,
    )
    .expect("write manifest");

    let out = agentpack_in(
        &home,
        &workspace,
        &[
            "--target",
            "claude_code",
            "deploy",
            "--apply",
            "--yes",
            "--json",
        ],
    );
    assert!(out.status.success());

    assert!(workspace.join(".claude/skills/my-skill/SKILL.md").exists());
    assert!(
        workspace
            .join(".claude/skills/.agentpack.manifest.json")
            .exists()
    );
}
