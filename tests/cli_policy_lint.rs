use std::path::Path;
use std::process::Command;

fn agentpack(args: &[&str]) -> std::process::Output {
    let tmp = tempfile::tempdir().expect("tempdir");
    agentpack_in(tmp.path(), args)
}

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
fn policy_lint_json_succeeds_when_no_violations() {
    let td = tempfile::tempdir().expect("tempdir");
    let repo = td.path();

    std::fs::create_dir_all(repo.join(".codex/skills/test-skill")).expect("mkdir");
    std::fs::write(
        repo.join(".codex/skills/test-skill/SKILL.md"),
        r#"---
name: test-skill
description: test skill
---

# test-skill
"#,
    )
    .expect("write SKILL.md");

    std::fs::create_dir_all(repo.join(".claude/commands")).expect("mkdir");
    std::fs::write(
        repo.join(".claude/commands/ap-test.md"),
        r#"---
description: "test command"
allowed-tools:
  - Bash("agentpack plan --json")
---

!bash
agentpack plan --json
"#,
    )
    .expect("write command");

    let out = agentpack(&["--repo", repo.to_str().unwrap(), "policy", "lint", "--json"]);
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "policy.lint");
    assert_eq!(v["data"]["summary"]["violations"], 0);
    assert!(v["data"]["issues"].as_array().is_some());
}

#[test]
fn policy_lint_json_fails_with_policy_violations() {
    let td = tempfile::tempdir().expect("tempdir");
    let repo = td.path();

    std::fs::create_dir_all(repo.join(".codex/skills/bad")).expect("mkdir");
    std::fs::write(
        repo.join(".codex/skills/bad/SKILL.md"),
        "# no frontmatter\n",
    )
    .expect("write bad SKILL.md");

    std::fs::create_dir_all(repo.join(".claude/commands")).expect("mkdir");
    std::fs::write(
        repo.join(".claude/commands/ap-bad.md"),
        r#"---
description: "bad command"
---

!bash
agentpack deploy --apply --json
"#,
    )
    .expect("write bad command");

    let out = agentpack(&["--repo", repo.to_str().unwrap(), "policy", "lint", "--json"]);
    assert!(!out.status.success());

    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_POLICY_VIOLATIONS");

    let issues = v["errors"][0]["details"]["issues"]
        .as_array()
        .expect("issues array");
    assert!(issues.len() >= 2);
    assert!(issues.iter().any(|i| i["rule"] == "skill_frontmatter"));
    assert!(
        issues
            .iter()
            .any(|i| i["rule"] == "claude_command_allowed_tools")
    );
    assert!(issues.iter().any(|i| i["rule"] == "dangerous_defaults"));
}

#[test]
fn policy_lint_json_fails_when_distribution_policy_requires_missing_target() {
    let td = tempfile::tempdir().expect("tempdir");
    let repo = td.path();

    std::fs::write(
        repo.join("agentpack.org.yaml"),
        r#"version: 1

distribution_policy:
  required_targets: ["codex"]
"#,
    )
    .expect("write agentpack.org.yaml");

    std::fs::write(
        repo.join("agentpack.yaml"),
        r#"version: 1

profiles:
  default:
    include_tags: []

targets:
  claude_code:
    mode: files
    scope: project
    options: {}

modules: []
"#,
    )
    .expect("write agentpack.yaml");

    let out = agentpack(&["--repo", repo.to_str().unwrap(), "policy", "lint", "--json"]);
    assert!(!out.status.success());

    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_POLICY_VIOLATIONS");

    let issues = v["errors"][0]["details"]["issues"]
        .as_array()
        .expect("issues array");
    assert!(
        issues
            .iter()
            .any(|i| i["rule"] == "distribution_required_targets")
    );
}

#[test]
fn policy_lint_json_fails_when_distribution_policy_requires_enabled_modules() {
    let td = tempfile::tempdir().expect("tempdir");
    let repo = td.path();

    std::fs::write(
        repo.join("agentpack.org.yaml"),
        r#"version: 1

distribution_policy:
  required_modules: ["instructions:base"]
"#,
    )
    .expect("write agentpack.org.yaml");

    std::fs::write(
        repo.join("agentpack.yaml"),
        r#"version: 1

profiles:
  default:
    include_tags: []

targets:
  codex:
    mode: files
    scope: user
    options: {}

modules:
  - id: instructions:base
    type: instructions
    enabled: false
    targets: ["codex"]
    source:
      local_path:
        path: "modules/instructions/base"
"#,
    )
    .expect("write agentpack.yaml");

    let out = agentpack(&["--repo", repo.to_str().unwrap(), "policy", "lint", "--json"]);
    assert!(!out.status.success());

    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_POLICY_VIOLATIONS");

    let issues = v["errors"][0]["details"]["issues"]
        .as_array()
        .expect("issues array");

    let required_modules = issues
        .iter()
        .find(|i| i["rule"] == "distribution_required_modules")
        .expect("distribution_required_modules issue");

    let disabled = required_modules["details"]["disabled"]
        .as_array()
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    assert!(
        disabled
            .iter()
            .any(|v| v.as_str() == Some("instructions:base"))
    );
}
