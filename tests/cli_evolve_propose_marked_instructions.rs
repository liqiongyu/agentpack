use std::path::Path;
use std::process::Command;

fn agentpack_in(home: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .output()
        .expect("run agentpack")
}

fn agentpack_in_cwd(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .output()
        .expect("run agentpack")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

#[test]
fn evolve_propose_can_map_marked_instructions_sections_to_modules() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    let repo_dir = tmp.path().join("repo");
    let i1 = repo_dir.join("modules/instructions/one");
    let i2 = repo_dir.join("modules/instructions/two");
    std::fs::create_dir_all(&i1).expect("create instructions module 1");
    std::fs::create_dir_all(&i2).expect("create instructions module 2");
    std::fs::write(i1.join("AGENTS.md"), "# one\n").expect("write AGENTS 1");
    std::fs::write(i2.join("AGENTS.md"), "# two\n").expect("write AGENTS 2");

    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: ["instructions:one","instructions:two"]
    exclude_modules: []

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_agents_global: true
      write_agents_repo_root: false
      write_user_prompts: false
      write_user_skills: false
      write_repo_skills: false

modules:
  - id: "instructions:one"
    type: instructions
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/instructions/one"
  - id: "instructions:two"
    type: instructions
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/instructions/two"
"#,
        codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");

    let deploy = agentpack_in(
        tmp.path(),
        &["--target", "codex", "deploy", "--apply", "--json", "--yes"],
    );
    assert!(deploy.status.success());

    let agents_path = codex_home.join("AGENTS.md");
    let mut agents = std::fs::read_to_string(&agents_path).expect("read AGENTS.md");
    assert!(agents.contains("<!-- agentpack:module=instructions:one -->"));
    assert!(agents.contains("<!-- agentpack:module=instructions:two -->"));
    assert!(agents.contains("<!-- /agentpack -->"));

    agents = agents.replace("# one", "# one edited");
    std::fs::write(&agents_path, agents).expect("write drifted AGENTS.md");

    let out = agentpack_in(
        tmp.path(),
        &[
            "--target",
            "codex",
            "evolve",
            "propose",
            "--dry-run",
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "evolve.propose");
    assert_eq!(v["data"]["reason"], "dry_run");

    let candidates = v["data"]["candidates"]
        .as_array()
        .expect("candidates array");
    assert!(
        candidates
            .iter()
            .any(|c| c["module_id"] == "instructions:one")
    );

    let skipped = v["data"]["skipped"].as_array().expect("skipped array");
    assert!(!skipped.iter().any(|s| s["reason"] == "multi_module_output"));
}

#[test]
fn evolve_propose_can_map_marked_instructions_sections_to_modules_for_vscode() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("create workspace");

    assert!(
        agentpack_in_cwd(tmp.path(), &workspace, &["init"])
            .status
            .success()
    );

    let repo_dir = tmp.path().join("repo");
    let i1 = repo_dir.join("modules/instructions/one");
    let i2 = repo_dir.join("modules/instructions/two");
    std::fs::create_dir_all(&i1).expect("create instructions module 1");
    std::fs::create_dir_all(&i2).expect("create instructions module 2");
    std::fs::write(i1.join("AGENTS.md"), "# one\n").expect("write AGENTS 1");
    std::fs::write(i2.join("AGENTS.md"), "# two\n").expect("write AGENTS 2");

    let manifest = r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: ["instructions:one","instructions:two"]
    exclude_modules: []

targets:
  vscode:
    mode: files
    scope: project
    options:
      write_instructions: true
      write_prompts: false

modules:
  - id: "instructions:one"
    type: instructions
    enabled: true
    tags: []
    targets: ["vscode"]
    source:
      local_path:
        path: "modules/instructions/one"
  - id: "instructions:two"
    type: instructions
    enabled: true
    tags: []
    targets: ["vscode"]
    source:
      local_path:
        path: "modules/instructions/two"
"#;
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");

    let deploy = agentpack_in_cwd(
        tmp.path(),
        &workspace,
        &["--target", "vscode", "deploy", "--apply", "--json", "--yes"],
    );
    assert!(deploy.status.success());

    let agents_path = workspace.join(".github/copilot-instructions.md");
    let mut agents = std::fs::read_to_string(&agents_path).expect("read copilot-instructions.md");
    assert!(agents.contains("<!-- agentpack:module=instructions:one -->"));
    assert!(agents.contains("<!-- agentpack:module=instructions:two -->"));
    assert!(agents.contains("<!-- /agentpack -->"));

    agents = agents.replace("# one", "# one edited");
    std::fs::write(&agents_path, agents).expect("write drifted copilot-instructions.md");

    let out = agentpack_in_cwd(
        tmp.path(),
        &workspace,
        &[
            "--target",
            "vscode",
            "evolve",
            "propose",
            "--dry-run",
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "evolve.propose");
    assert_eq!(v["data"]["reason"], "dry_run");

    let candidates = v["data"]["candidates"]
        .as_array()
        .expect("candidates array");
    assert!(
        candidates
            .iter()
            .any(|c| c["module_id"] == "instructions:one")
    );

    let skipped = v["data"]["skipped"].as_array().expect("skipped array");
    assert!(!skipped.iter().any(|s| s["reason"] == "multi_module_output"));
}

#[test]
fn evolve_propose_can_map_marked_instructions_sections_to_modules_for_jetbrains() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("create workspace");

    assert!(
        agentpack_in_cwd(tmp.path(), &workspace, &["init"])
            .status
            .success()
    );

    let repo_dir = tmp.path().join("repo");
    let i1 = repo_dir.join("modules/instructions/one");
    let i2 = repo_dir.join("modules/instructions/two");
    std::fs::create_dir_all(&i1).expect("create instructions module 1");
    std::fs::create_dir_all(&i2).expect("create instructions module 2");
    std::fs::write(i1.join("AGENTS.md"), "# one\n").expect("write AGENTS 1");
    std::fs::write(i2.join("AGENTS.md"), "# two\n").expect("write AGENTS 2");

    let manifest = r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: ["instructions:one","instructions:two"]
    exclude_modules: []

targets:
  jetbrains:
    mode: files
    scope: project
    options:
      write_guidelines: true

modules:
  - id: "instructions:one"
    type: instructions
    enabled: true
    tags: []
    targets: ["jetbrains"]
    source:
      local_path:
        path: "modules/instructions/one"
  - id: "instructions:two"
    type: instructions
    enabled: true
    tags: []
    targets: ["jetbrains"]
    source:
      local_path:
        path: "modules/instructions/two"
"#;
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");

    let deploy = agentpack_in_cwd(
        tmp.path(),
        &workspace,
        &[
            "--target",
            "jetbrains",
            "deploy",
            "--apply",
            "--json",
            "--yes",
        ],
    );
    assert!(deploy.status.success());

    let guidelines_path = workspace.join(".junie/guidelines.md");
    let mut guidelines = std::fs::read_to_string(&guidelines_path).expect("read guidelines.md");
    assert!(guidelines.contains("<!-- agentpack:module=instructions:one -->"));
    assert!(guidelines.contains("<!-- agentpack:module=instructions:two -->"));
    assert!(guidelines.contains("<!-- /agentpack -->"));

    guidelines = guidelines.replace("# one", "# one edited");
    std::fs::write(&guidelines_path, guidelines).expect("write drifted guidelines.md");

    let out = agentpack_in_cwd(
        tmp.path(),
        &workspace,
        &[
            "--target",
            "jetbrains",
            "evolve",
            "propose",
            "--dry-run",
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "evolve.propose");
    assert_eq!(v["data"]["reason"], "dry_run");

    let candidates = v["data"]["candidates"]
        .as_array()
        .expect("candidates array");
    assert!(
        candidates
            .iter()
            .any(|c| c["module_id"] == "instructions:one")
    );

    let skipped = v["data"]["skipped"].as_array().expect("skipped array");
    assert!(!skipped.iter().any(|s| s["reason"] == "multi_module_output"));
}

#[test]
fn evolve_propose_can_map_marked_instructions_sections_to_modules_for_zed() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("create workspace");

    assert!(
        agentpack_in_cwd(tmp.path(), &workspace, &["init"])
            .status
            .success()
    );

    let repo_dir = tmp.path().join("repo");
    let i1 = repo_dir.join("modules/instructions/one");
    let i2 = repo_dir.join("modules/instructions/two");
    std::fs::create_dir_all(&i1).expect("create instructions module 1");
    std::fs::create_dir_all(&i2).expect("create instructions module 2");
    std::fs::write(i1.join("AGENTS.md"), "# one\n").expect("write AGENTS 1");
    std::fs::write(i2.join("AGENTS.md"), "# two\n").expect("write AGENTS 2");

    let manifest = r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: ["instructions:one","instructions:two"]
    exclude_modules: []

targets:
  zed:
    mode: files
    scope: project
    options:
      write_rules: true

modules:
  - id: "instructions:one"
    type: instructions
    enabled: true
    tags: []
    targets: ["zed"]
    source:
      local_path:
        path: "modules/instructions/one"
  - id: "instructions:two"
    type: instructions
    enabled: true
    tags: []
    targets: ["zed"]
    source:
      local_path:
        path: "modules/instructions/two"
"#;
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");

    let deploy = agentpack_in_cwd(
        tmp.path(),
        &workspace,
        &["--target", "zed", "deploy", "--apply", "--json", "--yes"],
    );
    assert!(deploy.status.success());

    let rules_path = workspace.join(".rules");
    let mut rules = std::fs::read_to_string(&rules_path).expect("read .rules");
    assert!(rules.contains("<!-- agentpack:module=instructions:one -->"));
    assert!(rules.contains("<!-- agentpack:module=instructions:two -->"));
    assert!(rules.contains("<!-- /agentpack -->"));

    rules = rules.replace("# one", "# one edited");
    std::fs::write(&rules_path, rules).expect("write drifted .rules");

    let out = agentpack_in_cwd(
        tmp.path(),
        &workspace,
        &[
            "--target",
            "zed",
            "evolve",
            "propose",
            "--dry-run",
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "evolve.propose");
    assert_eq!(v["data"]["reason"], "dry_run");

    let candidates = v["data"]["candidates"]
        .as_array()
        .expect("candidates array");
    assert!(
        candidates
            .iter()
            .any(|c| c["module_id"] == "instructions:one")
    );

    let skipped = v["data"]["skipped"].as_array().expect("skipped array");
    assert!(!skipped.iter().any(|s| s["reason"] == "multi_module_output"));
}
