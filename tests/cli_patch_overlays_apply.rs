use std::path::Path;
use std::process::Command;

use agentpack::ids::module_fs_key;

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .env("HOME", home)
        .env("EDITOR", "")
        .output()
        .expect("run agentpack")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn init_git(dir: &Path) {
    assert!(
        Command::new("git")
            .current_dir(dir)
            .args(["init"])
            .output()
            .expect("git init")
            .status
            .success()
    );
}

#[test]
fn patch_overlays_apply_during_deploy() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let workspace = home.join("workspace");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    init_git(&workspace);

    let init = agentpack_in(home, &workspace, &["init"]);
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

    let overlay_dir = repo_dir
        .join("overlays")
        .join(module_fs_key("skill:my-skill"));
    std::fs::create_dir_all(overlay_dir.join(".agentpack/patches")).expect("create patch dir");
    std::fs::write(
        overlay_dir.join(".agentpack/overlay.json"),
        "{\n  \"overlay_kind\": \"patch\"\n}\n",
    )
    .expect("write overlay meta");
    std::fs::write(
        overlay_dir.join(".agentpack/patches/SKILL.md.patch"),
        "--- a/SKILL.md\n+++ b/SKILL.md\n@@ -1,6 +1,6 @@\n ---\n name: my-skill\n-description: Example Skill for tests\n+description: Patched Skill for tests\n ---\n \n # my-skill\n",
    )
    .expect("write patch");

    let out = agentpack_in(
        home,
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
    assert!(out.status.success(), "deploy should succeed");

    let deployed = std::fs::read_to_string(workspace.join(".claude/skills/my-skill/SKILL.md"))
        .expect("read deployed skill");
    assert!(
        deployed.contains("description: Patched Skill for tests"),
        "expected patched content"
    );
}

#[test]
fn patch_overlay_apply_failure_returns_stable_error_code() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let workspace = home.join("workspace");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    init_git(&workspace);

    let init = agentpack_in(home, &workspace, &["init"]);
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

    let overlay_dir = repo_dir
        .join("overlays")
        .join(module_fs_key("skill:my-skill"));
    std::fs::create_dir_all(overlay_dir.join(".agentpack/patches")).expect("create patch dir");
    std::fs::write(
        overlay_dir.join(".agentpack/overlay.json"),
        "{\n  \"overlay_kind\": \"patch\"\n}\n",
    )
    .expect("write overlay meta");
    // Wrong context line to force apply failure.
    std::fs::write(
        overlay_dir.join(".agentpack/patches/SKILL.md.patch"),
        "--- a/SKILL.md\n+++ b/SKILL.md\n@@ -1,6 +1,6 @@\n ---\n name: my-skill\n-description: DOES NOT MATCH\n+description: Patched Skill for tests\n ---\n \n # my-skill\n",
    )
    .expect("write patch");

    let out = agentpack_in(
        home,
        &workspace,
        &["--target", "claude_code", "plan", "--json"],
    );
    assert!(!out.status.success(), "plan should fail on patch apply");

    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_OVERLAY_PATCH_APPLY_FAILED");
}
