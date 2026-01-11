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
fn json_mode_requires_yes_for_mutating_commands() {
    let cases: &[(&str, &[&str])] = &[
        ("init", &["init", "--json"]),
        (
            "add",
            &["add", "skill", "local:modules/skill_test", "--json"],
        ),
        ("remove", &["remove", "skill:test", "--json"]),
        ("lock", &["lock", "--json"]),
        ("fetch", &["fetch", "--json"]),
        (
            "remote set",
            &[
                "remote",
                "set",
                "https://example.invalid/repo.git",
                "--json",
            ],
        ),
        ("sync", &["sync", "--rebase", "--json"]),
        ("record", &["record", "--json"]),
        ("overlay edit", &["overlay", "edit", "skill:test", "--json"]),
        (
            "rollback",
            &["rollback", "--to", "snapshot-does-not-matter", "--json"],
        ),
    ];

    for (name, args) in cases {
        let output = run_agentpack(args);
        assert!(!output.status.success(), "case {name} should fail");

        let v = parse_stdout_json(&output);
        assert_eq!(v["ok"], false, "case {name} ok=false");
        assert_eq!(
            v["errors"][0]["code"], "E_CONFIRM_REQUIRED",
            "case {name} error code"
        );
    }
}
