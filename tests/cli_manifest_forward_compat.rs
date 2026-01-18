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
fn unsupported_target_manifest_schema_version_is_non_fatal_for_status_and_plan() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(codex_home.join("prompts")).expect("create codex prompts dir");

    let repo_dir = tmp.path().join("repo");
    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: []
    exclude_modules: []

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_agents_global: false
      write_agents_repo_root: false
      write_user_prompts: true
      write_user_skills: false
      write_repo_skills: false

modules: []
"#,
        codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");

    std::fs::write(
        codex_home.join("prompts/.agentpack.manifest.codex.json"),
        r#"{"schema_version": 999}"#,
    )
    .expect("write unsupported manifest");

    let status = agentpack_in(tmp.path(), &["--target", "codex", "status", "--json"]);
    assert!(status.status.success());
    let v = parse_stdout_json(&status);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "status");
    let warnings = v["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| w
            .as_str()
            .unwrap_or_default()
            .contains("unsupported schema_version")),
        "warnings include unsupported schema_version"
    );

    let plan = agentpack_in(tmp.path(), &["--target", "codex", "plan", "--json"]);
    assert!(plan.status.success());
    let v = parse_stdout_json(&plan);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "plan");
    let warnings = v["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| w
            .as_str()
            .unwrap_or_default()
            .contains("unsupported schema_version")),
        "warnings include unsupported schema_version"
    );
}
