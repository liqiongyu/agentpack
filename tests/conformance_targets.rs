mod conformance_harness;

use conformance_harness::ConformanceHarness;
use std::path::{Path, PathBuf};

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

#[cfg(feature = "target-codex")]
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

#[cfg(feature = "target-claude-code")]
fn write_manifest_claude_code(repo_dir: &Path) {
    let manifest = r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  claude_code:
    mode: files
    scope: project
    options:
      write_repo_commands: true
      write_user_commands: false

modules:
  - id: command:hello
    type: command
    source:
      local_path:
        path: modules/claude-commands/hello
    enabled: true
    tags: ["base"]
    targets: ["claude_code"]
"#;
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");
}

#[cfg(feature = "target-cursor")]
fn write_manifest_cursor(repo_dir: &Path) {
    let manifest = r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  cursor:
    mode: files
    scope: project
    options:
      write_rules: true

modules:
  - id: instructions:base
    type: instructions
    source:
      local_path:
        path: modules/instructions/base
    enabled: true
    tags: ["base"]
    targets: ["cursor"]
"#;
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");
}

#[cfg(feature = "target-vscode")]
fn write_manifest_vscode(repo_dir: &Path) {
    let manifest = r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  vscode:
    mode: files
    scope: project
    options:
      write_instructions: true
      write_prompts: true

modules:
  - id: instructions:base
    type: instructions
    source:
      local_path:
        path: modules/instructions/base
    enabled: true
    tags: ["base"]
    targets: ["vscode"]
  - id: prompt:hello
    type: prompt
    source:
      local_path:
        path: modules/prompts/hello
    enabled: true
    tags: ["base"]
    targets: ["vscode"]
"#;
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");
}

#[cfg(feature = "target-jetbrains")]
fn write_manifest_jetbrains(repo_dir: &Path) {
    let manifest = r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  jetbrains:
    mode: files
    scope: project
    options:
      write_guidelines: true

modules:
  - id: instructions:base
    type: instructions
    source:
      local_path:
        path: modules/instructions/base
    enabled: true
    tags: ["base"]
    targets: ["jetbrains"]
"#;
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

#[cfg(feature = "target-codex")]
#[test]
fn conformance_codex_smoke() {
    let harness = ConformanceHarness::new();
    let home = harness.home();
    let workspace = harness.workspace();

    let init = harness.agentpack(&["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    let codex_home = workspace.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");
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

    // First deployment: manifests exist.
    let deploy1 = harness.agentpack(&["--target", "codex", "deploy", "--apply", "--yes", "--json"]);
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

    let status = harness.agentpack(&["--target", "codex", "status", "--json"]);
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
    let deploy_fix =
        harness.agentpack(&["--target", "codex", "deploy", "--apply", "--yes", "--json"]);
    assert!(deploy_fix.status.success());
    assert!(unmanaged.exists());

    // Rollback: change desired outputs via module edit, deploy again, then rollback.
    write_module(
        &repo_dir,
        "modules/prompts/hello",
        "hello.md",
        "Hello prompt v2\n",
    );
    let deploy2 = harness.agentpack(&["--target", "codex", "deploy", "--apply", "--yes", "--json"]);
    assert!(deploy2.status.success());
    assert_eq!(
        std::fs::read_to_string(&deployed_prompt).expect("read deployed prompt"),
        "Hello prompt v2\n"
    );

    let rollback = harness.agentpack(&[
        "--target",
        "codex",
        "rollback",
        "--to",
        snapshot1.as_str(),
        "--yes",
        "--json",
    ]);
    assert!(rollback.status.success());
    let rollback_json = parse_stdout_json(&rollback);
    assert_envelope_shape(&rollback_json, "rollback", true);
    assert_eq!(
        std::fs::read_to_string(&deployed_prompt).expect("read deployed prompt"),
        v1
    );
    assert!(unmanaged.exists());
}

#[cfg(feature = "target-claude-code")]
#[test]
fn conformance_claude_code_smoke() {
    let harness = ConformanceHarness::new();
    let home = harness.home();
    let workspace = harness.workspace();

    let init = harness.agentpack(&["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    write_manifest_claude_code(&repo_dir);

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

    let deploy1 = harness.agentpack(&[
        "--target",
        "claude_code",
        "deploy",
        "--apply",
        "--yes",
        "--json",
    ]);
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

    let status = harness.agentpack(&["--target", "claude_code", "status", "--json"]);
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

    let deploy_fix = harness.agentpack(&[
        "--target",
        "claude_code",
        "deploy",
        "--apply",
        "--yes",
        "--json",
    ]);
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
    let deploy2 = harness.agentpack(&[
        "--target",
        "claude_code",
        "deploy",
        "--apply",
        "--yes",
        "--json",
    ]);
    assert!(deploy2.status.success());
    assert!(
        std::fs::read_to_string(&deployed_cmd)
            .expect("read deployed command")
            .contains("Hello v2")
    );

    let rollback = harness.agentpack(&[
        "--target",
        "claude_code",
        "rollback",
        "--to",
        snapshot1.as_str(),
        "--yes",
        "--json",
    ]);
    assert!(rollback.status.success());
    let rollback_json = parse_stdout_json(&rollback);
    assert_envelope_shape(&rollback_json, "rollback", true);
    assert_eq!(
        std::fs::read_to_string(&deployed_cmd).expect("read deployed command"),
        v1
    );
    assert!(unmanaged.exists());
}

#[cfg(feature = "target-cursor")]
#[test]
fn conformance_cursor_smoke() {
    let harness = ConformanceHarness::new();
    let home = harness.home();
    let workspace = harness.workspace();

    let init = harness.agentpack(&["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    write_manifest_cursor(&repo_dir);

    write_module(
        &repo_dir,
        "modules/instructions/base",
        "AGENTS.md",
        "# Base instructions\n",
    );

    let deploy1 =
        harness.agentpack(&["--target", "cursor", "deploy", "--apply", "--yes", "--json"]);
    assert!(
        deploy1.status.success(),
        "deploy failed: status={:?}\nstdout={}\nstderr={}",
        deploy1.status.code(),
        String::from_utf8_lossy(&deploy1.stdout),
        String::from_utf8_lossy(&deploy1.stderr)
    );
    let deploy1_json = parse_stdout_json(&deploy1);
    assert_envelope_shape(&deploy1_json, "deploy", true);
    let snapshot1 = deploy1_json["data"]["snapshot_id"]
        .as_str()
        .expect("snapshot_id")
        .to_string();

    let rules_dir = workspace.join(".cursor").join("rules");
    assert!(rules_dir.join(".agentpack.manifest.json").exists());

    let changes = deploy1_json["data"]["changes"]
        .as_array()
        .expect("changes array");
    let deployed_rule = changes
        .iter()
        .filter_map(|c| c["path"].as_str())
        .map(PathBuf::from)
        .find(|p| p.extension().and_then(|s| s.to_str()) == Some("mdc"))
        .expect("deployed rule path");
    assert!(
        deployed_rule.exists(),
        "deployed rule missing at {}; files={:?}",
        deployed_rule.display(),
        list_all_files(&rules_dir)
    );
    let v1 = std::fs::read_to_string(&deployed_rule).expect("read deployed rule");

    let unmanaged = rules_dir.join("unmanaged.mdc");
    std::fs::write(&unmanaged, "unmanaged\n").expect("write unmanaged");
    std::fs::write(&deployed_rule, "local drift\n").expect("write drift");

    let status = harness.agentpack(&["--target", "cursor", "status", "--json"]);
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

    let deploy_fix =
        harness.agentpack(&["--target", "cursor", "deploy", "--apply", "--yes", "--json"]);
    assert!(deploy_fix.status.success());
    assert!(unmanaged.exists());

    write_module(
        &repo_dir,
        "modules/instructions/base",
        "AGENTS.md",
        "# Base instructions v2\n",
    );
    let deploy2 =
        harness.agentpack(&["--target", "cursor", "deploy", "--apply", "--yes", "--json"]);
    assert!(deploy2.status.success());
    assert!(
        std::fs::read_to_string(&deployed_rule)
            .expect("read deployed rule")
            .contains("Base instructions v2")
    );

    let rollback = harness.agentpack(&[
        "--target",
        "cursor",
        "rollback",
        "--to",
        snapshot1.as_str(),
        "--yes",
        "--json",
    ]);
    assert!(rollback.status.success());
    let rollback_json = parse_stdout_json(&rollback);
    assert_envelope_shape(&rollback_json, "rollback", true);
    assert_eq!(
        std::fs::read_to_string(&deployed_rule).expect("read deployed rule"),
        v1
    );
    assert!(unmanaged.exists());
}

#[cfg(feature = "target-vscode")]
#[test]
fn conformance_vscode_smoke() {
    let harness = ConformanceHarness::new();
    let home = harness.home();
    let workspace = harness.workspace();

    let init = harness.agentpack(&["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    write_manifest_vscode(&repo_dir);

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

    let deploy1 =
        harness.agentpack(&["--target", "vscode", "deploy", "--apply", "--yes", "--json"]);
    assert!(
        deploy1.status.success(),
        "deploy failed: status={:?}\nstdout={}\nstderr={}",
        deploy1.status.code(),
        String::from_utf8_lossy(&deploy1.stdout),
        String::from_utf8_lossy(&deploy1.stderr)
    );
    let deploy1_json = parse_stdout_json(&deploy1);
    assert_envelope_shape(&deploy1_json, "deploy", true);
    let snapshot1 = deploy1_json["data"]["snapshot_id"]
        .as_str()
        .expect("snapshot_id")
        .to_string();

    let github_dir = workspace.join(".github");
    let prompts_dir = github_dir.join("prompts");
    assert!(github_dir.join(".agentpack.manifest.json").exists());
    assert!(prompts_dir.join(".agentpack.manifest.json").exists());

    let instructions_path = github_dir.join("copilot-instructions.md");
    let prompt_path = prompts_dir.join("hello.prompt.md");
    assert!(
        instructions_path.exists(),
        "missing {}; files={:?}",
        instructions_path.display(),
        list_all_files(&github_dir)
    );
    assert!(
        prompt_path.exists(),
        "missing {}; files={:?}",
        prompt_path.display(),
        list_all_files(&prompts_dir)
    );
    let v1 = std::fs::read_to_string(&prompt_path).expect("read deployed prompt");

    let unmanaged = prompts_dir.join("unmanaged.prompt.md");
    std::fs::write(&unmanaged, "unmanaged\n").expect("write unmanaged");
    std::fs::write(&prompt_path, "local drift\n").expect("write drift");

    let status = harness.agentpack(&["--target", "vscode", "status", "--json"]);
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

    let deploy_fix =
        harness.agentpack(&["--target", "vscode", "deploy", "--apply", "--yes", "--json"]);
    assert!(deploy_fix.status.success());
    assert!(unmanaged.exists());

    write_module(
        &repo_dir,
        "modules/prompts/hello",
        "hello.md",
        "Hello prompt v2\n",
    );
    let deploy2 =
        harness.agentpack(&["--target", "vscode", "deploy", "--apply", "--yes", "--json"]);
    assert!(deploy2.status.success());
    assert!(
        std::fs::read_to_string(&prompt_path)
            .expect("read deployed prompt")
            .contains("Hello prompt v2")
    );

    let rollback = harness.agentpack(&[
        "--target",
        "vscode",
        "rollback",
        "--to",
        snapshot1.as_str(),
        "--yes",
        "--json",
    ]);
    assert!(rollback.status.success());
    let rollback_json = parse_stdout_json(&rollback);
    assert_envelope_shape(&rollback_json, "rollback", true);
    assert_eq!(
        std::fs::read_to_string(&prompt_path).expect("read deployed prompt"),
        v1
    );
    assert!(unmanaged.exists());
}

#[cfg(feature = "target-jetbrains")]
#[test]
fn conformance_jetbrains_smoke() {
    let harness = ConformanceHarness::new();
    let home = harness.home();
    let workspace = harness.workspace();

    let init = harness.agentpack(&["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    write_manifest_jetbrains(&repo_dir);

    write_module(
        &repo_dir,
        "modules/instructions/base",
        "AGENTS.md",
        "# Base instructions\n",
    );

    let deploy1 = harness.agentpack(&[
        "--target",
        "jetbrains",
        "deploy",
        "--apply",
        "--yes",
        "--json",
    ]);
    assert!(
        deploy1.status.success(),
        "deploy failed: status={:?}\nstdout={}\nstderr={}",
        deploy1.status.code(),
        String::from_utf8_lossy(&deploy1.stdout),
        String::from_utf8_lossy(&deploy1.stderr)
    );
    let deploy1_json = parse_stdout_json(&deploy1);
    assert_envelope_shape(&deploy1_json, "deploy", true);
    let snapshot1 = deploy1_json["data"]["snapshot_id"]
        .as_str()
        .expect("snapshot_id")
        .to_string();

    let junie_dir = workspace.join(".junie");
    assert!(junie_dir.join(".agentpack.manifest.json").exists());

    let guidelines_path = junie_dir.join("guidelines.md");
    assert!(
        guidelines_path.exists(),
        "missing {}; files={:?}",
        guidelines_path.display(),
        list_all_files(&junie_dir)
    );
    let v1 = std::fs::read_to_string(&guidelines_path).expect("read deployed guidelines");

    let unmanaged = junie_dir.join("unmanaged.md");
    std::fs::write(&unmanaged, "unmanaged\n").expect("write unmanaged");
    std::fs::write(&guidelines_path, "local drift\n").expect("write drift");

    let status = harness.agentpack(&["--target", "jetbrains", "status", "--json"]);
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

    let deploy_fix = harness.agentpack(&[
        "--target",
        "jetbrains",
        "deploy",
        "--apply",
        "--yes",
        "--json",
    ]);
    assert!(deploy_fix.status.success());
    assert!(unmanaged.exists());

    write_module(
        &repo_dir,
        "modules/instructions/base",
        "AGENTS.md",
        "# Base instructions v2\n",
    );
    let deploy2 = harness.agentpack(&[
        "--target",
        "jetbrains",
        "deploy",
        "--apply",
        "--yes",
        "--json",
    ]);
    assert!(deploy2.status.success());
    assert!(
        std::fs::read_to_string(&guidelines_path)
            .expect("read deployed guidelines")
            .contains("Base instructions v2")
    );

    let rollback = harness.agentpack(&[
        "--target",
        "jetbrains",
        "rollback",
        "--to",
        snapshot1.as_str(),
        "--yes",
        "--json",
    ]);
    assert!(rollback.status.success());
    let rollback_json = parse_stdout_json(&rollback);
    assert_envelope_shape(&rollback_json, "rollback", true);
    assert_eq!(
        std::fs::read_to_string(&guidelines_path).expect("read deployed guidelines"),
        v1
    );
    assert!(unmanaged.exists());
}
