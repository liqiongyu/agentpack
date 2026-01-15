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
fn help_json_is_self_describing() {
    let output = run_agentpack(&["help", "--json"]);
    assert!(output.status.success());

    let v = parse_stdout_json(&output);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "help");
    assert!(v["data"]["global_args"].is_array());
    assert!(v["data"]["commands"].is_array());
    assert!(v["data"]["commands"].as_array().unwrap().len() > 5);
    assert!(
        v["data"]["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.get("supports_json").is_some())
    );
    assert!(
        v["data"]["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["id"] == "completions" && c["supports_json"] == false)
    );
    assert!(v["data"]["mutating_commands"].is_array());
    assert!(
        v["data"]["mutating_commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == "init")
    );
    assert!(
        v["data"]["mutating_commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == "deploy --apply")
    );

    assert!(v["data"]["targets"].is_array());
    let targets = v["data"]["targets"].as_array().unwrap();
    #[cfg(feature = "target-codex")]
    assert!(targets.iter().any(|t| t == "codex"));
    #[cfg(feature = "target-claude-code")]
    assert!(targets.iter().any(|t| t == "claude_code"));
    #[cfg(feature = "target-cursor")]
    assert!(targets.iter().any(|t| t == "cursor"));
    #[cfg(feature = "target-vscode")]
    assert!(targets.iter().any(|t| t == "vscode"));
}

#[test]
fn schema_json_includes_envelope_shape() {
    let output = run_agentpack(&["schema", "--json"]);
    assert!(output.status.success());

    let v = parse_stdout_json(&output);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "schema");
    assert_eq!(v["data"]["envelope"]["schema_version"], 1);
    assert!(v["data"]["envelope"]["fields"].is_object());
}
