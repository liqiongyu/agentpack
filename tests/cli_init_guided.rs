use std::path::Path;
use std::process::{Command, Stdio};

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .env("AGENTPACK_MACHINE_ID", "test-machine")
        .env("HOME", home)
        .env("USERPROFILE", home)
        .output()
        .expect("run agentpack")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn init_workspace(home: &Path) -> std::path::PathBuf {
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
    workspace
}

#[test]
fn init_guided_json_non_tty_returns_e_tty_required_and_does_not_write() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = env!("CARGO_BIN_EXE_agentpack");

    let out = Command::new(bin)
        .args(["init", "--guided", "--json"])
        .stdin(Stdio::null())
        .env("AGENTPACK_HOME", tmp.path())
        .env("HOME", tmp.path())
        .env("USERPROFILE", tmp.path())
        .output()
        .expect("run agentpack");

    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "init");
    assert_eq!(v["errors"][0]["code"], "E_TTY_REQUIRED");

    assert!(!tmp.path().join("repo").exists());
}

#[test]
#[cfg(feature = "target-codex")]
fn guided_manifest_is_usable_for_update_preview_deploy() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(home);

    assert!(agentpack_in(home, &workspace, &["init"]).status.success());

    let repo_dir = home.join("repo");
    let codex_home = home.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    std::fs::write(
        repo_dir.join("modules/instructions/base/AGENTS.md"),
        "# Base instructions (guided)\n",
    )
    .expect("write base AGENTS.md");

    // Represents the minimal manifest shape `init --guided` generates.
    // Use an explicit codex_home to keep the test sandboxed on all platforms.
    std::fs::write(
        repo_dir.join("agentpack.yaml"),
        format!(
            r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  codex:
    mode: files
    scope: both
    options:
      codex_home: '{codex_home}'
      write_repo_skills: true
      write_user_skills: true
      write_user_prompts: true
      write_agents_global: true
      write_agents_repo_root: true

modules:
  - id: instructions:base
    type: instructions
    tags: ["base"]
    targets: ["codex"]
    source:
      local_path:
        path: modules/instructions/base
"#,
            codex_home = codex_home.display()
        ),
    )
    .expect("write manifest");

    let update = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "update", "--json", "--yes"],
    );
    assert!(update.status.success());

    let preview = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "preview", "--diff"],
    );
    assert!(preview.status.success());

    let deploy = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "deploy", "--apply", "--yes", "--json"],
    );
    assert!(deploy.status.success());

    assert!(workspace.join("AGENTS.md").exists());
    assert!(codex_home.join("AGENTS.md").exists());

    let deploy_v = parse_stdout_json(&deploy);
    assert_eq!(deploy_v["data"]["applied"], true);
    assert!(
        !deploy_v["data"]["snapshot_id"]
            .as_str()
            .unwrap_or_default()
            .is_empty()
    );
}
