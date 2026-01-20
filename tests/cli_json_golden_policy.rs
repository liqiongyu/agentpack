use std::path::Path;
use std::process::Command;

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .env("AGENTPACK_MACHINE_ID", "test-machine")
        .env("HOME", home)
        .output()
        .expect("run agentpack")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn canonicalize_json(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut items: Vec<(String, serde_json::Value)> = map.into_iter().collect();
            items.sort_by(|a, b| a.0.cmp(&b.0));

            let mut out = serde_json::Map::new();
            for (k, v) in items {
                out.insert(k, canonicalize_json(v));
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(canonicalize_json).collect::<Vec<_>>())
        }
        other => other,
    }
}

fn normalize_json(value: serde_json::Value, tmp_prefix: &str) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(k, normalize_json(v, tmp_prefix));
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => serde_json::Value::Array(
            arr.into_iter()
                .map(|v| normalize_json(v, tmp_prefix))
                .collect::<Vec<_>>(),
        ),
        serde_json::Value::String(s) => {
            let mut out = s.replace('\\', "/");
            out = out.replace(tmp_prefix, "<TMP>");
            serde_json::Value::String(out)
        }
        other => other,
    }
}

fn pretty_json(value: serde_json::Value) -> String {
    let canonical = canonicalize_json(value);
    let mut out = serde_json::to_string_pretty(&canonical).expect("pretty json");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
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

fn assert_data_matches_golden(
    v: &serde_json::Value,
    golden_path: &str,
    tmp_prefix: &str,
) -> anyhow::Result<()> {
    let normalized = normalize_json(v["data"].clone(), tmp_prefix);
    let actual = pretty_json(normalized);

    let expected_path = std::path::Path::new(golden_path);
    if !expected_path.exists() {
        anyhow::bail!("missing golden snapshot: {golden_path}\n\n{actual}");
    }

    let expected = std::fs::read_to_string(expected_path)
        .map_err(|e| anyhow::anyhow!("read golden {golden_path}: {e}"))?;
    assert_eq!(actual, expected);
    Ok(())
}

fn write_file(path: &Path, contents: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("create {}: {e}", parent.display()))?;
    }
    std::fs::write(path, contents).map_err(|e| anyhow::anyhow!("write {}: {e}", path.display()))?;
    Ok(())
}

#[test]
fn json_policy_command_data_matches_golden_snapshots() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let cwd = home;

    let repo = home.join("repo");
    std::fs::create_dir_all(&repo)?;

    write_file(
        repo.join("policies/my-pack/rule.txt").as_path(),
        "deny: example\n",
    )?;
    write_file(
        repo.join("agentpack.org.yaml").as_path(),
        r#"version: 1

policy_pack:
  source: "local:policies/my-pack"
"#,
    )?;

    write_file(
        repo.join("skills/example-skill/SKILL.md").as_path(),
        r#"---
name: example-skill
description: Example skill for policy golden tests.
---

# Example
"#,
    )?;
    write_file(
        repo.join("modules/claude-commands/example.md").as_path(),
        r#"# example

This command does not use bash.
"#,
    )?;

    let tmp_prefix = home.to_string_lossy().replace('\\', "/");

    let lock = agentpack_in(home, cwd, &["policy", "lock", "--json", "--yes"]);
    assert!(lock.status.success());
    let lock_v = parse_stdout_json(&lock);
    assert_envelope_shape(&lock_v, "policy.lock", true);
    assert_data_matches_golden(
        &lock_v,
        "tests/golden/policy_lock_json_data.json",
        &tmp_prefix,
    )?;

    let lint = agentpack_in(home, cwd, &["policy", "lint", "--json"]);
    assert!(lint.status.success());
    let lint_v = parse_stdout_json(&lint);
    assert_envelope_shape(&lint_v, "policy.lint", true);
    assert_data_matches_golden(
        &lint_v,
        "tests/golden/policy_lint_json_data.json",
        &tmp_prefix,
    )?;

    Ok(())
}
