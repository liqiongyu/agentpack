#![cfg(windows)]

use std::path::{Path, PathBuf};

mod conformance_harness;

use conformance_harness::ConformanceHarness;

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn write_module(repo_dir: &Path, rel_dir: &str, filename: &str, content: &str) -> PathBuf {
    let dir = repo_dir.join(rel_dir);
    std::fs::create_dir_all(&dir).expect("create module dir");
    let path = dir.join(filename);
    std::fs::write(&path, content).expect("write module file");
    path
}

fn write_manifest_codex(repo_dir: &Path, codex_home: &Path) {
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
"#,
        codex_home = codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");
}

#[test]
fn deploy_json_returns_stable_code_for_invalid_path() {
    let harness = ConformanceHarness::new();
    let home = harness.home();
    let workspace = harness.workspace();

    let init = harness.agentpack(&["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    let codex_home = workspace.join("codex<home");
    write_manifest_codex(&repo_dir, &codex_home);

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

    let deploy = harness.agentpack(&["--target", "codex", "deploy", "--apply", "--yes", "--json"]);
    assert!(!deploy.status.success());
    let v = parse_stdout_json(&deploy);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_IO_INVALID_PATH");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("io_invalid_path")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["fix_path", "retry_command"])
    );
}

#[test]
fn deploy_json_returns_stable_code_for_path_too_long() {
    let harness = ConformanceHarness::new();
    let home = harness.home();
    let workspace = harness.workspace();

    let init = harness.agentpack(&["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");

    let mut codex_home = workspace.to_path_buf();
    // Build a path that exceeds Windows limits while keeping each component reasonable.
    for i in 0..250 {
        codex_home = codex_home.join(format!("d{i:03}{}", "a".repeat(200)));
    }

    write_manifest_codex(&repo_dir, &codex_home);

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

    let deploy = harness.agentpack(&["--target", "codex", "deploy", "--apply", "--yes", "--json"]);
    assert!(!deploy.status.success());
    let v = parse_stdout_json(&deploy);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_IO_PATH_TOO_LONG");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("io_path_too_long")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["shorten_path", "enable_long_paths", "retry_command"])
    );
}

#[test]
fn deploy_json_returns_stable_code_for_read_only_destination() {
    let harness = ConformanceHarness::new();
    let home = harness.home();
    let workspace = harness.workspace();

    let init = harness.agentpack(&["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    let codex_home = workspace.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    write_manifest_codex(&repo_dir, &codex_home);

    let prompt_module = write_module(
        &repo_dir,
        "modules/prompts/hello",
        "hello.md",
        "Hello prompt v1\n",
    );
    write_module(
        &repo_dir,
        "modules/instructions/base",
        "AGENTS.md",
        "# Base instructions\n",
    );

    let deploy1 = harness.agentpack(&["--target", "codex", "deploy", "--apply", "--yes", "--json"]);
    assert!(deploy1.status.success());

    let deployed_prompt = codex_home.join("prompts").join("hello.md");
    assert!(
        deployed_prompt.exists(),
        "expected {}",
        deployed_prompt.display()
    );
    let mut perms = std::fs::metadata(&deployed_prompt)
        .expect("stat deployed prompt")
        .permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&deployed_prompt, perms).expect("set read-only");

    std::fs::write(&prompt_module, "Hello prompt v2\n").expect("update module");
    let deploy2 = harness.agentpack(&["--target", "codex", "deploy", "--apply", "--yes", "--json"]);
    assert!(!deploy2.status.success());

    let v = parse_stdout_json(&deploy2);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_IO_PERMISSION_DENIED");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("io_permission_denied")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["fix_permissions", "retry_command"])
    );
}
