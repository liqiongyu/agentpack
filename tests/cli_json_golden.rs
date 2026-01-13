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

#[test]
fn help_json_data_matches_golden_snapshot() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let output = agentpack_in(tmp.path(), &["help", "--json"]);
    assert!(output.status.success());

    let v = parse_stdout_json(&output);
    assert_envelope_shape(&v, "help", true);

    let actual = pretty_json(v["data"].clone());
    let expected =
        std::fs::read_to_string("tests/golden/help_json_data.json").expect("read golden");
    assert_eq!(actual, expected);
}

#[test]
fn schema_json_data_matches_golden_snapshot() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let output = agentpack_in(tmp.path(), &["schema", "--json"]);
    assert!(output.status.success());

    let v = parse_stdout_json(&output);
    assert_envelope_shape(&v, "schema", true);

    let actual = pretty_json(v["data"].clone());
    let expected =
        std::fs::read_to_string("tests/golden/schema_json_data.json").expect("read golden");
    assert_eq!(actual, expected);
}

#[test]
fn json_error_envelope_has_required_fields() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let output = agentpack_in(tmp.path(), &["init", "--json"]);
    assert!(!output.status.success());

    let v = parse_stdout_json(&output);
    assert_envelope_shape(&v, "init", false);
    assert_eq!(v["errors"][0]["code"], "E_CONFIRM_REQUIRED");
}
