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
fn policy_lock_json_errors_when_policy_config_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = agentpack_in(tmp.path(), &["policy", "lock", "--json", "--yes"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_POLICY_CONFIG_MISSING");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("policy_config_missing")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["create_policy_config", "retry_command"])
    );
}

#[test]
fn policy_lock_json_errors_when_policy_config_unsupported_version() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("repo")).expect("mkdir repo");
    std::fs::write(
        tmp.path().join("repo").join("agentpack.org.yaml"),
        r#"version: 2

policy_pack:
  source: "local:policies/my-pack"
"#,
    )
    .expect("write org config");

    let out = agentpack_in(tmp.path(), &["policy", "lock", "--json", "--yes"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(
        v["errors"][0]["code"],
        "E_POLICY_CONFIG_UNSUPPORTED_VERSION"
    );
    assert_eq!(v["errors"][0]["details"]["version"], 2);
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("policy_config_unsupported_version")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!([
            "upgrade_agentpack",
            "fix_policy_config_version",
            "retry_command"
        ])
    );
}

#[test]
fn policy_lock_writes_deterministic_lockfile_for_local_pack() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(repo.join("policies/my-pack")).expect("mkdir pack");
    std::fs::write(repo.join("policies/my-pack/rule.txt"), "hello").expect("write pack file");

    std::fs::write(
        repo.join("agentpack.org.yaml"),
        r#"version: 1

policy_pack:
  source: "local:policies/my-pack"
"#,
    )
    .expect("write org config");

    assert!(
        agentpack_in(tmp.path(), &["policy", "lock"])
            .status
            .success()
    );
    let lockfile_path = repo.join("agentpack.org.lock.json");
    let first = std::fs::read_to_string(&lockfile_path).expect("read lockfile");

    assert!(
        agentpack_in(tmp.path(), &["policy", "lock"])
            .status
            .success()
    );
    let second = std::fs::read_to_string(&lockfile_path).expect("read lockfile 2");
    assert_eq!(first, second);

    let lock: serde_json::Value = serde_json::from_str(&first).expect("lockfile json");
    assert_eq!(lock["version"], 1);
    assert_eq!(
        lock["policy_pack"]["source"]["local_path"]["path"],
        "policies/my-pack"
    );
    assert_eq!(lock["policy_pack"]["resolved_version"], "local");
    assert!(lock["policy_pack"]["sha256"].as_str().is_some());
    assert!(
        lock["policy_pack"]["file_manifest"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["path"] == "rule.txt")
    );
}

#[test]
fn policy_lint_fails_when_policy_pack_configured_without_lock() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(repo.join("policies/my-pack")).expect("mkdir pack");
    std::fs::write(repo.join("policies/my-pack/rule.txt"), "hello").expect("write pack file");
    std::fs::write(
        repo.join("agentpack.org.yaml"),
        r#"version: 1

policy_pack:
  source: "local:policies/my-pack"
"#,
    )
    .expect("write org config");

    let out = agentpack_in(tmp.path(), &["policy", "lint", "--json"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_POLICY_VIOLATIONS");
    let issues = v["errors"][0]["details"]["issues"]
        .as_array()
        .expect("issues array");
    assert!(issues.iter().any(|i| i["rule"] == "policy_pack_lock"));
}

#[test]
fn policy_lint_succeeds_after_policy_lock() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(repo.join("policies/my-pack")).expect("mkdir pack");
    std::fs::write(repo.join("policies/my-pack/rule.txt"), "hello").expect("write pack file");
    std::fs::write(
        repo.join("agentpack.org.yaml"),
        r#"version: 1

policy_pack:
  source: "local:policies/my-pack"
"#,
    )
    .expect("write org config");

    assert!(
        agentpack_in(tmp.path(), &["policy", "lock"])
            .status
            .success()
    );
    let out = agentpack_in(tmp.path(), &["policy", "lint", "--json"]);
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "policy.lint");
}

#[test]
fn policy_lock_json_errors_when_policy_pack_remote_is_not_allowlisted() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let repo = tmp.path().join("repo");
    std::fs::write(
        repo.join("agentpack.org.yaml"),
        r#"version: 1

policy_pack:
  source: "git:file:///does/not/exist/policy-pack.git#ref=v1.0.0"

supply_chain_policy:
  allowed_git_remotes: ["github.com/acme/"]
"#,
    )
    .expect("write org config");

    let out = agentpack_in(tmp.path(), &["policy", "lock", "--json", "--yes"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_POLICY_CONFIG_INVALID");
    assert_eq!(
        v["errors"][0]["details"]["remote"],
        "file:///does/not/exist/policy-pack.git"
    );
    let allowed = v["errors"][0]["details"]["allowed_git_remotes"]
        .as_array()
        .expect("allowed_git_remotes array");
    assert!(
        allowed
            .iter()
            .any(|v| v.as_str() == Some("github.com/acme/"))
    );
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("policy_config_invalid")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["fix_policy_config", "retry_command"])
    );
}
