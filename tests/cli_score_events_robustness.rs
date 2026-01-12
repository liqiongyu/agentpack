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
fn score_json_skips_malformed_lines_with_warnings() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let logs_dir = tmp.path().join("state").join("logs");
    std::fs::create_dir_all(&logs_dir).expect("create logs dir");

    let events_path = logs_dir.join("events.jsonl");
    let good = serde_json::json!({
        "schema_version": 1,
        "recorded_at": "2026-01-11T00:00:00Z",
        "machine_id": "m1",
        "event": { "module_id": "skill:test", "success": false }
    });

    let unsupported = serde_json::json!({
        "schema_version": 999,
        "recorded_at": "2026-01-11T00:00:01Z",
        "machine_id": "m1",
        "event": { "module_id": "skill:ignored", "success": true }
    });

    let content = format!(
        "{}\nnot-json\n{}\n",
        serde_json::to_string(&good).unwrap(),
        serde_json::to_string(&unsupported).unwrap()
    );
    std::fs::write(&events_path, content).expect("write events.jsonl");

    let out = agentpack_in(tmp.path(), &["score", "--json"]);
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert!(v["warnings"].is_array());
    assert!(v["warnings"].as_array().unwrap().len() >= 2);

    let modules = v["data"]["modules"].as_array().expect("modules array");
    let item = modules
        .iter()
        .find(|m| m["module_id"] == "skill:test")
        .expect("skill:test present");
    assert_eq!(item["failures"], 1);
    assert_eq!(item["total"], 1);
}
