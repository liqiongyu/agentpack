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
fn json_mode_requires_yes_for_add() {
    let output = run_agentpack(&["add", "skill", "local:modules/skill_test", "--json"]);
    assert!(!output.status.success());

    let v = parse_stdout_json(&output);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "add");
    assert_eq!(v["errors"][0]["code"], "E_CONFIRM_REQUIRED");
}

#[test]
fn json_mode_requires_yes_for_fetch() {
    let output = run_agentpack(&["fetch", "--json"]);
    assert!(!output.status.success());

    let v = parse_stdout_json(&output);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "fetch");
    assert_eq!(v["errors"][0]["code"], "E_CONFIRM_REQUIRED");
}
