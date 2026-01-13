use std::path::{Path, PathBuf};
use std::process::Command;

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
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

fn assert_envelope_shape(v: &serde_json::Value, expected_command: &str, ok: bool) {
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["ok"], ok);
    assert_eq!(v["command"], expected_command);
    assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
    assert!(v["data"].is_object());
    assert!(v["warnings"].is_array());
    assert!(v["errors"].is_array());
}

fn write_module(repo_dir: &Path, rel_dir: &str, filename: &str, content: &str) -> PathBuf {
    let dir = repo_dir.join(rel_dir);
    std::fs::create_dir_all(&dir).expect("create module dir");
    let path = dir.join(filename);
    std::fs::write(&path, content).expect("write module file");
    path
}

fn write_manifest(repo_dir: &Path, codex_home: &Path) {
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
      write_agents_global: true
      write_agents_repo_root: false
      write_user_prompts: true
      write_user_skills: false
      write_repo_skills: false
  claude_code:
    mode: files
    scope: project
    options:
      write_repo_commands: true
      write_user_commands: false

modules:
  - id: instructions:base
    type: instructions
    source:
      local_path:
        path: modules/instructions/base
    enabled: true
    tags: ["base"]
    targets: ["codex"]
  - id: prompt:hello
    type: prompt
    source:
      local_path:
        path: modules/prompts/hello
    enabled: true
    tags: ["base"]
    targets: ["codex"]
  - id: command:hello
    type: command
    source:
      local_path:
        path: modules/claude-commands/hello
    enabled: true
    tags: ["base"]
    targets: ["claude_code"]
"#,
        codex_home = codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");
}

fn list_all_files(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let path = e.path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.push(path.to_string_lossy().to_string());
            }
        }
    }

    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

#[test]
fn conformance_codex_smoke() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = tmp.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    assert!(git_in(&workspace, &["init"]).status.success());

    let init = agentpack_in(home, &workspace, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    let codex_home = workspace.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");
    write_manifest(&repo_dir, &codex_home);

    write_module(
        &repo_dir,
        "modules/instructions/base",
        "AGENTS.md",
        "# Base instructions\n",
    );
    write_module(
        &repo_dir,
        "modules/prompts/hello",
        "hello.md",
        "Hello prompt v1\n",
    );

    // First deployment: manifests exist.
    let deploy1 = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "deploy", "--apply", "--yes", "--json"],
    );
    assert!(deploy1.status.success());
    let deploy1_json = parse_stdout_json(&deploy1);
    assert_envelope_shape(&deploy1_json, "deploy", true);
    let snapshot1 = deploy1_json["data"]["snapshot_id"]
        .as_str()
        .expect("snapshot_id")
        .to_string();

    assert!(codex_home.join(".agentpack.manifest.json").exists());
    assert!(
        codex_home
            .join("prompts")
            .join(".agentpack.manifest.json")
            .exists()
    );

    // Drift classification: modified + extra.
    let changes = deploy1_json["data"]["changes"]
        .as_array()
        .expect("changes array");
    let deployed_prompt = changes
        .iter()
        .filter_map(|c| c["path"].as_str())
        .map(PathBuf::from)
        .find(|p| p.file_name().and_then(|s| s.to_str()) == Some("hello.md"))
        .expect("deployed prompt path");
    assert!(
        deployed_prompt.exists(),
        "deployed prompt missing at {}; files={:?}",
        deployed_prompt.display(),
        list_all_files(&codex_home)
    );
    let v1 = std::fs::read_to_string(&deployed_prompt).expect("read deployed prompt");

    let unmanaged = codex_home.join("prompts").join("unmanaged.txt");
    std::fs::write(&unmanaged, "unmanaged\n").expect("write unmanaged");
    std::fs::write(&deployed_prompt, "local drift\n").expect("write drift");

    let status = agentpack_in(home, &workspace, &["--target", "codex", "status", "--json"]);
    assert!(status.status.success());
    let status_json = parse_stdout_json(&status);
    assert_envelope_shape(&status_json, "status", true);
    let drift = status_json["data"]["drift"]
        .as_array()
        .expect("drift array");
    assert!(drift.iter().any(|d| d["kind"] == "modified"));
    assert!(drift.iter().any(|d| d["kind"] == "extra"));
    let summary = &status_json["data"]["summary"];
    assert!(summary["modified"].as_u64().unwrap_or(0) >= 1);
    assert!(summary["extra"].as_u64().unwrap_or(0) >= 1);

    // Safe apply: should not delete unmanaged files.
    let deploy_fix = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "deploy", "--apply", "--yes", "--json"],
    );
    assert!(deploy_fix.status.success());
    assert!(unmanaged.exists());

    // Rollback: change desired outputs via module edit, deploy again, then rollback.
    write_module(
        &repo_dir,
        "modules/prompts/hello",
        "hello.md",
        "Hello prompt v2\n",
    );
    let deploy2 = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "deploy", "--apply", "--yes", "--json"],
    );
    assert!(deploy2.status.success());
    assert_eq!(
        std::fs::read_to_string(&deployed_prompt).expect("read deployed prompt"),
        "Hello prompt v2\n"
    );

    let rollback = agentpack_in(
        home,
        &workspace,
        &[
            "--target",
            "codex",
            "rollback",
            "--to",
            snapshot1.as_str(),
            "--yes",
            "--json",
        ],
    );
    assert!(rollback.status.success());
    let rollback_json = parse_stdout_json(&rollback);
    assert_envelope_shape(&rollback_json, "rollback", true);
    assert_eq!(
        std::fs::read_to_string(&deployed_prompt).expect("read deployed prompt"),
        v1
    );
    assert!(unmanaged.exists());
}

#[test]
fn conformance_claude_code_smoke() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = tmp.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    assert!(git_in(&workspace, &["init"]).status.success());

    let init = agentpack_in(home, &workspace, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    let codex_home = workspace.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");
    write_manifest(&repo_dir, &codex_home);

    write_module(
        &repo_dir,
        "modules/claude-commands/hello",
        "hello.md",
        r#"---
description: "Hello command"
allowed-tools:
  - Bash("echo hi")
---

Hello v1
"#,
    );

    let deploy1 = agentpack_in(
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
    assert!(deploy1.status.success());
    let deploy1_json = parse_stdout_json(&deploy1);
    assert_envelope_shape(&deploy1_json, "deploy", true);
    let snapshot1 = deploy1_json["data"]["snapshot_id"]
        .as_str()
        .expect("snapshot_id")
        .to_string();

    let commands_dir = workspace.join(".claude").join("commands");
    assert!(commands_dir.join(".agentpack.manifest.json").exists());

    let changes = deploy1_json["data"]["changes"]
        .as_array()
        .expect("changes array");
    let deployed_cmd = changes
        .iter()
        .filter_map(|c| c["path"].as_str())
        .map(PathBuf::from)
        .find(|p| p.file_name().and_then(|s| s.to_str()) == Some("hello.md"))
        .expect("deployed command path");
    let commands_dir = workspace.join(".claude").join("commands");
    assert!(
        deployed_cmd.exists(),
        "deployed command missing at {}; files={:?}",
        deployed_cmd.display(),
        list_all_files(&commands_dir)
    );
    let v1 = std::fs::read_to_string(&deployed_cmd).expect("read deployed command");

    let unmanaged = commands_dir.join("unmanaged.txt");
    std::fs::write(&unmanaged, "unmanaged\n").expect("write unmanaged");
    std::fs::write(&deployed_cmd, "local drift\n").expect("write drift");

    let status = agentpack_in(
        home,
        &workspace,
        &["--target", "claude_code", "status", "--json"],
    );
    assert!(status.status.success());
    let status_json = parse_stdout_json(&status);
    assert_envelope_shape(&status_json, "status", true);
    let drift = status_json["data"]["drift"]
        .as_array()
        .expect("drift array");
    assert!(drift.iter().any(|d| d["kind"] == "modified"));
    assert!(drift.iter().any(|d| d["kind"] == "extra"));
    let summary = &status_json["data"]["summary"];
    assert!(summary["modified"].as_u64().unwrap_or(0) >= 1);
    assert!(summary["extra"].as_u64().unwrap_or(0) >= 1);

    let deploy_fix = agentpack_in(
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
    assert!(deploy_fix.status.success());
    assert!(unmanaged.exists());

    write_module(
        &repo_dir,
        "modules/claude-commands/hello",
        "hello.md",
        r#"---
description: "Hello command"
allowed-tools:
  - Bash("echo hi")
---

Hello v2
"#,
    );
    let deploy2 = agentpack_in(
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
    assert!(deploy2.status.success());
    assert!(
        std::fs::read_to_string(&deployed_cmd)
            .expect("read deployed command")
            .contains("Hello v2")
    );

    let rollback = agentpack_in(
        home,
        &workspace,
        &[
            "--target",
            "claude_code",
            "rollback",
            "--to",
            snapshot1.as_str(),
            "--yes",
            "--json",
        ],
    );
    assert!(rollback.status.success());
    let rollback_json = parse_stdout_json(&rollback);
    assert_envelope_shape(&rollback_json, "rollback", true);
    assert_eq!(
        std::fs::read_to_string(&deployed_cmd).expect("read deployed command"),
        v1
    );
    assert!(unmanaged.exists());
}
