use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{Command, Stdio};

fn read_json_line(reader: &mut BufReader<std::process::ChildStdout>) -> serde_json::Value {
    let mut line = String::new();
    let n = reader.read_line(&mut line).expect("read line");
    assert!(n > 0, "unexpected EOF from mcp server");
    serde_json::from_str(&line).expect("line is valid json")
}

#[test]
fn mcp_server_stdio_mutating_tools_require_approval_and_work_when_approved() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo_dir = tmp.path().join("repo");
    std::fs::create_dir_all(repo_dir.join("modules/instructions/base")).expect("create module dir");

    std::fs::write(
        repo_dir.join("modules/instructions/base/AGENTS.md"),
        "hello from agentpack\n",
    )
    .expect("write AGENTS.md");

    let codex_home = tmp.path().join("codex");
    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_agents_global: true
      write_agents_repo_root: false
      write_user_skills: false
      write_repo_skills: false
      write_user_prompts: false

modules:
  - id: instructions:base
    type: instructions
    tags: ["base"]
    source:
      local_path:
        path: "modules/instructions/base"
"#,
        codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write agentpack.yaml");

    let bin = env!("CARGO_BIN_EXE_agentpack");
    let mut child = Command::new(bin)
        .args(["mcp", "serve"])
        .env("AGENTPACK_HOME", tmp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn agentpack mcp serve");

    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut stdout = BufReader::new(stdout);

    // initialize
    stdin
        .write_all(
            br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"agentpack-test","version":"0.0.0"}}}"#,
        )
        .expect("write initialize");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");
    let _init = read_json_line(&mut stdout);

    // initialized notification
    stdin
        .write_all(br#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#)
        .expect("write initialized");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    // deploy returns confirm_token (stage 1)
    stdin
        .write_all(
            br#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"deploy","arguments":{}}}"#,
        )
        .expect("write tools/call deploy");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    let deploy_plan = read_json_line(&mut stdout);
    assert_eq!(deploy_plan["id"], 2);
    assert!(!deploy_plan["result"]["isError"].as_bool().unwrap_or(false));
    let deploy_plan_text = deploy_plan["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");
    let deploy_plan_env: serde_json::Value =
        serde_json::from_str(deploy_plan_text).expect("envelope json");
    let confirm_token = deploy_plan_env["data"]["confirm_token"]
        .as_str()
        .expect("confirm_token string")
        .to_string();

    // deploy_apply requires approval
    stdin
        .write_all(
            br#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"deploy_apply","arguments":{}}}"#,
        )
        .expect("write tools/call deploy_apply (no approval)");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    let deploy_no = read_json_line(&mut stdout);
    assert_eq!(deploy_no["id"], 3);
    assert!(deploy_no["result"]["isError"].as_bool().unwrap_or(false));
    let deploy_no_text = deploy_no["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");
    let deploy_no_env: serde_json::Value =
        serde_json::from_str(deploy_no_text).expect("envelope json");
    assert_eq!(deploy_no_env["command"], "deploy");
    assert_eq!(deploy_no_env["errors"][0]["code"], "E_CONFIRM_REQUIRED");

    assert!(
        !codex_home.join("AGENTS.md").exists(),
        "should not write without approval"
    );

    // deploy_apply with approval but without confirm_token is refused
    stdin
        .write_all(
            br#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"deploy_apply","arguments":{"yes":true}}}"#,
        )
        .expect("write tools/call deploy_apply (approved, missing confirm_token)");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    let deploy_missing_token = read_json_line(&mut stdout);
    assert_eq!(deploy_missing_token["id"], 4);
    assert!(
        deploy_missing_token["result"]["isError"]
            .as_bool()
            .unwrap_or(false)
    );
    let deploy_missing_token_text = deploy_missing_token["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");
    let deploy_missing_token_env: serde_json::Value =
        serde_json::from_str(deploy_missing_token_text).expect("envelope json");
    assert_eq!(deploy_missing_token_env["command"], "deploy");
    assert_eq!(
        deploy_missing_token_env["errors"][0]["code"],
        "E_CONFIRM_TOKEN_REQUIRED"
    );

    assert!(
        !codex_home.join("AGENTS.md").exists(),
        "should not write without confirm_token"
    );

    // deploy_apply with approval but bad confirm_token is refused
    stdin
        .write_all(
            br#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"deploy_apply","arguments":{"yes":true,"confirm_token":"not-a-real-token"}}}"#,
        )
        .expect("write tools/call deploy_apply (approved, bad confirm_token)");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    let deploy_bad_token = read_json_line(&mut stdout);
    assert_eq!(deploy_bad_token["id"], 5);
    assert!(
        deploy_bad_token["result"]["isError"]
            .as_bool()
            .unwrap_or(false)
    );
    let deploy_bad_token_text = deploy_bad_token["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");
    let deploy_bad_token_env: serde_json::Value =
        serde_json::from_str(deploy_bad_token_text).expect("envelope json");
    assert_eq!(deploy_bad_token_env["command"], "deploy");
    assert_eq!(
        deploy_bad_token_env["errors"][0]["code"],
        "E_CONFIRM_TOKEN_MISMATCH"
    );

    // deploy_apply with approval and correct confirm_token
    let deploy_apply_req = format!(
        r#"{{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{{"name":"deploy_apply","arguments":{{"yes":true,"confirm_token":"{}"}}}}}}"#,
        confirm_token
    );
    stdin
        .write_all(deploy_apply_req.as_bytes())
        .expect("write tools/call deploy_apply (approved)");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    let deploy_ok = read_json_line(&mut stdout);
    assert_eq!(deploy_ok["id"], 6);
    assert!(!deploy_ok["result"]["isError"].as_bool().unwrap_or(false));
    let deploy_ok_text = deploy_ok["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");
    let deploy_ok_env: serde_json::Value =
        serde_json::from_str(deploy_ok_text).expect("envelope json");
    assert_eq!(deploy_ok_env["command"], "deploy");
    assert!(deploy_ok_env["ok"].as_bool().unwrap_or(false));
    let snapshot_id = deploy_ok_env["data"]["snapshot_id"]
        .as_str()
        .expect("snapshot_id string")
        .to_string();

    assert!(
        codex_home.join("AGENTS.md").exists(),
        "deploy should write target outputs"
    );

    // rollback requires approval
    stdin
        .write_all(
            br#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"rollback","arguments":{"to":"ignored","yes":false}}}"#,
        )
        .expect("write tools/call rollback (no approval)");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    let rollback_no = read_json_line(&mut stdout);
    assert_eq!(rollback_no["id"], 7);
    assert!(rollback_no["result"]["isError"].as_bool().unwrap_or(false));
    let rollback_no_text = rollback_no["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");
    let rollback_no_env: serde_json::Value =
        serde_json::from_str(rollback_no_text).expect("envelope json");
    assert_eq!(rollback_no_env["command"], "rollback");
    assert_eq!(rollback_no_env["errors"][0]["code"], "E_CONFIRM_REQUIRED");

    // rollback with approval
    let rollback_req = format!(
        r#"{{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{{"name":"rollback","arguments":{{"to":"{}","yes":true}}}}}}"#,
        snapshot_id
    );
    stdin
        .write_all(rollback_req.as_bytes())
        .expect("write tools/call rollback (approved)");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    let rollback_ok = read_json_line(&mut stdout);
    assert_eq!(rollback_ok["id"], 8);
    assert!(!rollback_ok["result"]["isError"].as_bool().unwrap_or(false));
    let rollback_ok_text = rollback_ok["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");
    let rollback_ok_env: serde_json::Value =
        serde_json::from_str(rollback_ok_text).expect("envelope json");
    assert_eq!(rollback_ok_env["command"], "rollback");
    assert!(rollback_ok_env["ok"].as_bool().unwrap_or(false));

    let _ = child.kill();
    let _ = child.wait();
}
