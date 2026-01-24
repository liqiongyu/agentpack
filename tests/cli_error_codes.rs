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
fn json_error_code_config_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    std::fs::remove_file(tmp.path().join("repo").join("agentpack.yaml")).expect("remove manifest");

    let out = agentpack_in(tmp.path(), &["plan", "--json"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_CONFIG_MISSING");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("config_missing")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["run_init", "retry_with_repo", "retry_command"])
    );
}

#[test]
fn json_error_code_config_invalid_yaml() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    std::fs::write(
        tmp.path().join("repo").join("agentpack.yaml"),
        "version: [\n",
    )
    .expect("write invalid manifest");

    let out = agentpack_in(tmp.path(), &["plan", "--json"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_CONFIG_INVALID");
}

#[test]
fn json_error_code_config_unsupported_version() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let manifest = r#"version: 2

profiles:
  default:
    include_tags: []

targets:
  codex:
    mode: files
    scope: user
    options: {}

modules: []
"#;
    std::fs::write(tmp.path().join("repo").join("agentpack.yaml"), manifest)
        .expect("write manifest");

    let out = agentpack_in(tmp.path(), &["plan", "--json"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_CONFIG_UNSUPPORTED_VERSION");
}

#[test]
fn json_error_code_lockfile_missing_for_fetch() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let out = agentpack_in(tmp.path(), &["fetch", "--json", "--yes"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_LOCKFILE_MISSING");
}

#[test]
fn json_error_code_lockfile_invalid_for_fetch() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    std::fs::write(
        tmp.path().join("repo").join("agentpack.lock.json"),
        "{not json",
    )
    .expect("write invalid lockfile");

    let out = agentpack_in(tmp.path(), &["fetch", "--json", "--yes"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_LOCKFILE_INVALID");
}

#[test]
fn json_error_code_lockfile_unsupported_version_for_fetch() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    std::fs::write(
        tmp.path().join("repo").join("agentpack.lock.json"),
        r#"{"version":2,"generated_at":"t","modules":[]}"#,
    )
    .expect("write lockfile");

    let out = agentpack_in(tmp.path(), &["fetch", "--json", "--yes"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_LOCKFILE_UNSUPPORTED_VERSION");
}

#[test]
fn json_error_code_target_unsupported() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let out = agentpack_in(tmp.path(), &["--target", "nope", "plan", "--json"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_TARGET_UNSUPPORTED");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("target_filter_unsupported")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["inspect_help_json", "retry_with_supported_target"])
    );
}

#[test]
fn json_error_code_target_unsupported_in_manifest_includes_guidance() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    std::fs::write(
        tmp.path().join("repo").join("agentpack.yaml"),
        r#"version: 1

profiles:
  default:
    include_tags: []

targets:
  nope:
    mode: files
    scope: user
    options: {}

modules: []
"#,
    )
    .expect("write manifest");

    let out = agentpack_in(tmp.path(), &["plan", "--json"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_TARGET_UNSUPPORTED");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("target_not_compiled")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!([
            "inspect_help_json",
            "edit_manifest_targets",
            "rebuild_with_target_feature",
            "retry_command"
        ])
    );
}
