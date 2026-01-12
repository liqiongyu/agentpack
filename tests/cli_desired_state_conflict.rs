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

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

#[test]
fn plan_json_fails_with_stable_code_on_desired_state_conflict() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let init = agentpack_in(tmp.path(), &["init"]);
    assert!(init.status.success());

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    let repo_dir = tmp.path().join("repo");
    let m1 = repo_dir.join("modules/prompt_one");
    let m2 = repo_dir.join("modules/prompt_two");
    std::fs::create_dir_all(&m1).expect("create module 1");
    std::fs::create_dir_all(&m2).expect("create module 2");
    std::fs::write(m1.join("prompt.md"), "# one\n").expect("write prompt 1");
    std::fs::write(m2.join("prompt.md"), "# two\n").expect("write prompt 2");

    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: ["prompt:one","prompt:two"]
    exclude_modules: []

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_user_prompts: true
      write_user_skills: false
      write_repo_skills: false
      write_agents_global: false
      write_agents_repo_root: false

modules:
  - id: "prompt:one"
    type: prompt
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/prompt_one"
  - id: "prompt:two"
    type: prompt
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/prompt_two"
"#,
        codex_home.display()
    );
    let manifest_path = repo_dir.join("agentpack.yaml");
    std::fs::write(&manifest_path, manifest).expect("write manifest");

    let plan = agentpack_in(tmp.path(), &["--target", "codex", "plan", "--json"]);
    assert!(!plan.status.success());

    let v = parse_stdout_json(&plan);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_DESIRED_STATE_CONFLICT");
}

#[test]
fn plan_human_includes_conflict_path_and_module_ids() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let init = agentpack_in(tmp.path(), &["init"]);
    assert!(init.status.success());

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    let repo_dir = tmp.path().join("repo");
    let m1 = repo_dir.join("modules/prompt_one");
    let m2 = repo_dir.join("modules/prompt_two");
    std::fs::create_dir_all(&m1).expect("create module 1");
    std::fs::create_dir_all(&m2).expect("create module 2");
    std::fs::write(m1.join("prompt.md"), "# one\n").expect("write prompt 1");
    std::fs::write(m2.join("prompt.md"), "# two\n").expect("write prompt 2");

    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: ["prompt:one","prompt:two"]
    exclude_modules: []

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_user_prompts: true
      write_user_skills: false
      write_repo_skills: false
      write_agents_global: false
      write_agents_repo_root: false

modules:
  - id: "prompt:one"
    type: prompt
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/prompt_one"
  - id: "prompt:two"
    type: prompt
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/prompt_two"
"#,
        codex_home.display()
    );
    let manifest_path = repo_dir.join("agentpack.yaml");
    std::fs::write(&manifest_path, manifest).expect("write manifest");

    let plan = agentpack_in(tmp.path(), &["--target", "codex", "plan"]);
    assert!(!plan.status.success());

    let stderr = String::from_utf8_lossy(&plan.stderr);
    assert!(stderr.contains("E_DESIRED_STATE_CONFLICT"));
    assert!(stderr.contains("prompt:one"));
    assert!(stderr.contains("prompt:two"));

    let expected_suffix = std::path::Path::new("codex_home")
        .join("prompts")
        .join("prompt.md")
        .to_string_lossy()
        .to_string();
    assert!(stderr.contains(&expected_suffix));
}
