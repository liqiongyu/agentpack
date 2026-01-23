use std::process::Command;

fn run_agentpack(args: &[&str]) -> std::process::Output {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .args(args)
        .env("AGENTPACK_HOME", tmp.path())
        .output()
        .expect("run agentpack")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

#[test]
fn json_success_includes_command_id_and_command_path() {
    let output = run_agentpack(&["schema", "--json"]);
    assert!(output.status.success());

    let v = parse_stdout_json(&output);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "schema");
    assert_eq!(v["command_id"], "schema");
    assert_eq!(v["command_path"], serde_json::json!(["schema"]));
}

#[test]
fn json_error_includes_command_id_and_command_path_for_subcommand() {
    let output = run_agentpack(&[
        "remote",
        "set",
        "https://example.invalid/repo.git",
        "--json",
    ]);
    assert!(!output.status.success());

    let v = parse_stdout_json(&output);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "remote");
    assert_eq!(v["command_id"], "remote set");
    assert_eq!(v["command_path"], serde_json::json!(["remote", "set"]));
    assert_eq!(v["errors"][0]["code"], "E_CONFIRM_REQUIRED");
}

#[test]
fn json_error_includes_command_id_and_command_path_for_mutating_flag_variant() {
    let output = run_agentpack(&["doctor", "--fix", "--json"]);
    assert!(!output.status.success());

    let v = parse_stdout_json(&output);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "doctor");
    assert_eq!(v["command_id"], "doctor --fix");
    assert_eq!(v["command_path"], serde_json::json!(["doctor", "--fix"]));
    assert_eq!(v["errors"][0]["code"], "E_CONFIRM_REQUIRED");
}
