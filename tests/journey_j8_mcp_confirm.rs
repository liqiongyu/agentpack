mod journeys;

use std::io::{BufRead as _, BufReader, Write as _};
use std::path::Path;
use std::process::{Command, Stdio};

use journeys::common::TestEnv;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(path, contents).expect("write file");
}

fn read_json_line(reader: &mut BufReader<std::process::ChildStdout>) -> serde_json::Value {
    let mut line = String::new();
    let n = reader.read_line(&mut line).expect("read line");
    assert!(n > 0, "unexpected EOF from mcp server");
    serde_json::from_str(&line).expect("line is valid json")
}

fn mcp_initialize(
    stdin: &mut std::process::ChildStdin,
    stdout: &mut BufReader<std::process::ChildStdout>,
) {
    // initialize
    stdin
        .write_all(
            br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"agentpack-test","version":"0.0.0"}}}"#,
        )
        .expect("write initialize");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");
    let _init = read_json_line(stdout);

    // initialized notification
    stdin
        .write_all(br#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#)
        .expect("write initialized");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");
}

fn call_tool(
    stdin: &mut std::process::ChildStdin,
    stdout: &mut BufReader<std::process::ChildStdout>,
    id: u32,
    name: &str,
    args_json: &str,
) -> (serde_json::Value, serde_json::Value) {
    let req = format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"{name}","arguments":{args_json}}}}}"#
    );
    stdin.write_all(req.as_bytes()).expect("write tools/call");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    let res = read_json_line(stdout);
    assert_eq!(res["jsonrpc"], "2.0");
    assert_eq!(res["id"], id);
    let text = res["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");
    let envelope: serde_json::Value = serde_json::from_str(text).expect("envelope json");
    (res, envelope)
}

fn spawn_mcp_server(env: &TestEnv) -> std::process::Child {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_agentpack"));
    cmd.current_dir(env.workspace())
        .env("AGENTPACK_HOME", env.agentpack_home())
        .env("AGENTPACK_MACHINE_ID", journeys::common::TEST_MACHINE_ID)
        .env("HOME", env.home())
        .env("USERPROFILE", env.home())
        .env("CODEX_HOME", env.home().join(".codex"))
        .env("XDG_CONFIG_HOME", env.home())
        .env("XDG_CACHE_HOME", env.home())
        .env("XDG_DATA_HOME", env.home())
        .env("XDG_STATE_HOME", env.home())
        .env("EDITOR", "")
        .args(["mcp", "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    cmd.spawn().expect("spawn agentpack mcp serve")
}

#[test]
fn journey_j8_mcp_confirm_token_deploy_apply_and_rollback() {
    let env = TestEnv::new();
    env.init_repo();

    let repo_dir = env.repo_dir();
    let module_path = repo_dir.join("modules/instructions/base/AGENTS.md");
    write_file(&module_path, "v1\n");

    let codex_home = env.home().join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

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
      codex_home: '{codex_home}'
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
        codex_home = codex_home.display()
    );
    write_file(&env.manifest_path(), &manifest);

    let mut child = spawn_mcp_server(&env);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut stdout = BufReader::new(stdout);

    mcp_initialize(&mut stdin, &mut stdout);

    // Stage 1: deploy returns confirm_token.
    let (_deploy_res, deploy_env) = call_tool(&mut stdin, &mut stdout, 2, "deploy", "{}");
    assert_eq!(deploy_env["command"], "deploy");
    assert!(deploy_env["ok"].as_bool().unwrap_or(false));
    let confirm_token = deploy_env["data"]["confirm_token"]
        .as_str()
        .expect("confirm_token string")
        .to_string();
    assert!(!confirm_token.trim().is_empty());

    // deploy_apply without approval is refused.
    let (apply_no_res, apply_no_env) = call_tool(&mut stdin, &mut stdout, 3, "deploy_apply", "{}");
    assert!(apply_no_res["result"]["isError"].as_bool().unwrap_or(false));
    assert_eq!(apply_no_env["command"], "deploy");
    assert_eq!(apply_no_env["errors"][0]["code"], "E_CONFIRM_REQUIRED");

    // deploy_apply with approval but without confirm_token is refused.
    let (apply_missing_res, apply_missing_env) = call_tool(
        &mut stdin,
        &mut stdout,
        4,
        "deploy_apply",
        r#"{"yes":true}"#,
    );
    assert!(
        apply_missing_res["result"]["isError"]
            .as_bool()
            .unwrap_or(false)
    );
    assert_eq!(apply_missing_env["command"], "deploy");
    assert_eq!(
        apply_missing_env["errors"][0]["code"],
        "E_CONFIRM_TOKEN_REQUIRED"
    );

    // deploy_apply with approval but bad confirm_token is refused.
    let (apply_bad_res, apply_bad_env) = call_tool(
        &mut stdin,
        &mut stdout,
        5,
        "deploy_apply",
        r#"{"yes":true,"confirm_token":"not-a-real-token"}"#,
    );
    assert!(
        apply_bad_res["result"]["isError"]
            .as_bool()
            .unwrap_or(false)
    );
    assert_eq!(apply_bad_env["command"], "deploy");
    assert_eq!(
        apply_bad_env["errors"][0]["code"],
        "E_CONFIRM_TOKEN_MISMATCH"
    );

    // Mutate the repo after planning; apply with the old token must be refused due to plan mismatch.
    write_file(&module_path, "v2\n");
    let apply_mismatch_args = format!(r#"{{"yes":true,"confirm_token":"{confirm_token}"}}"#);
    let (apply_mismatch_res, apply_mismatch_env) = call_tool(
        &mut stdin,
        &mut stdout,
        6,
        "deploy_apply",
        apply_mismatch_args.as_str(),
    );
    assert!(
        apply_mismatch_res["result"]["isError"]
            .as_bool()
            .unwrap_or(false)
    );
    assert_eq!(apply_mismatch_env["command"], "deploy");
    assert_eq!(
        apply_mismatch_env["errors"][0]["code"],
        "E_CONFIRM_TOKEN_MISMATCH"
    );
    assert!(
        !codex_home.join("AGENTS.md").exists(),
        "should not write without a matching confirm_token"
    );

    // Apply v2 with a fresh token.
    let (_deploy_v2_res, deploy_v2_env) = call_tool(&mut stdin, &mut stdout, 7, "deploy", "{}");
    let token_v2 = deploy_v2_env["data"]["confirm_token"]
        .as_str()
        .expect("confirm_token string")
        .to_string();
    assert!(!token_v2.trim().is_empty());

    let apply_v2_args = format!(r#"{{"yes":true,"confirm_token":"{token_v2}"}}"#);
    let (_apply_v2_res, apply_v2_env) = call_tool(
        &mut stdin,
        &mut stdout,
        8,
        "deploy_apply",
        apply_v2_args.as_str(),
    );
    assert_eq!(apply_v2_env["command"], "deploy");
    assert!(apply_v2_env["ok"].as_bool().unwrap_or(false));
    let snapshot_v2 = apply_v2_env["data"]["snapshot_id"]
        .as_str()
        .expect("snapshot_id string")
        .to_string();
    assert!(!snapshot_v2.trim().is_empty());
    assert_eq!(
        std::fs::read_to_string(codex_home.join("AGENTS.md")).expect("read deployed AGENTS.md"),
        "v2\n"
    );

    // Apply v3 and then rollback to v2.
    write_file(&module_path, "v3\n");
    let (_deploy_v3_res, deploy_v3_env) = call_tool(&mut stdin, &mut stdout, 9, "deploy", "{}");
    let token_v3 = deploy_v3_env["data"]["confirm_token"]
        .as_str()
        .expect("confirm_token string")
        .to_string();
    assert!(!token_v3.trim().is_empty());
    let apply_v3_args = format!(r#"{{"yes":true,"confirm_token":"{token_v3}"}}"#);
    let (_apply_v3_res, apply_v3_env) = call_tool(
        &mut stdin,
        &mut stdout,
        10,
        "deploy_apply",
        apply_v3_args.as_str(),
    );
    assert!(apply_v3_env["ok"].as_bool().unwrap_or(false));
    assert_eq!(
        std::fs::read_to_string(codex_home.join("AGENTS.md")).expect("read deployed AGENTS.md"),
        "v3\n"
    );

    // rollback without approval is refused.
    let (rb_no_res, rb_no_env) = call_tool(
        &mut stdin,
        &mut stdout,
        11,
        "rollback",
        r#"{"to":"ignored","yes":false}"#,
    );
    assert!(rb_no_res["result"]["isError"].as_bool().unwrap_or(false));
    assert_eq!(rb_no_env["command"], "rollback");
    assert_eq!(rb_no_env["errors"][0]["code"], "E_CONFIRM_REQUIRED");

    // rollback with approval restores snapshot_v2.
    let rb_ok_args = format!(r#"{{"to":"{snapshot_v2}","yes":true}}"#);
    let (_rb_ok_res, rb_ok_env) = call_tool(&mut stdin, &mut stdout, 12, "rollback", &rb_ok_args);
    assert_eq!(rb_ok_env["command"], "rollback");
    assert!(rb_ok_env["ok"].as_bool().unwrap_or(false));
    assert_eq!(
        std::fs::read_to_string(codex_home.join("AGENTS.md")).expect("read deployed AGENTS.md"),
        "v2\n"
    );

    let _ = child.kill();
    let _ = child.wait();
}
