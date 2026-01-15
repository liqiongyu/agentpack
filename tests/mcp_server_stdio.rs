use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{Command, Stdio};

fn read_json_line(reader: &mut BufReader<std::process::ChildStdout>) -> serde_json::Value {
    let mut line = String::new();
    let n = reader.read_line(&mut line).expect("read line");
    assert!(n > 0, "unexpected EOF from mcp server");
    serde_json::from_str(&line).expect("line is valid json")
}

#[test]
fn mcp_server_stdio_handshake_and_tools_list_work() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo_dir = tmp.path().join("repo");
    std::fs::create_dir_all(&repo_dir).expect("create repo dir");

    let codex_home = tmp.path().join("codex");
    let manifest = format!(
        r#"version: 1

profiles:
  default: {{}}

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_repo_skills: false
      write_user_skills: true
      write_user_prompts: false
      write_agents_global: false
      write_agents_repo_root: false

modules: []
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

    let init = read_json_line(&mut stdout);
    assert_eq!(init["jsonrpc"], "2.0");
    assert_eq!(init["id"], 1);
    assert!(init["result"]["capabilities"].is_object());
    assert!(init["result"]["capabilities"]["tools"].is_object());
    assert_eq!(init["result"]["serverInfo"]["name"], "agentpack");

    // initialized notification
    stdin
        .write_all(br#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#)
        .expect("write initialized");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    // tools/list
    stdin
        .write_all(br#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#)
        .expect("write tools/list");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush");

    let tools_list = read_json_line(&mut stdout);
    assert_eq!(tools_list["jsonrpc"], "2.0");
    assert_eq!(tools_list["id"], 2);
    let tools = tools_list["result"]["tools"]
        .as_array()
        .expect("tools array");
    let names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().expect("tool name"))
        .collect();
    for required in [
        "plan",
        "diff",
        "status",
        "doctor",
        "deploy_apply",
        "rollback",
    ] {
        assert!(names.contains(&required), "missing tool: {required}");
    }

    for (idx, tool) in ["plan", "diff", "status", "doctor"].iter().enumerate() {
        let req_id = 3 + idx;
        let req = format!(
            r#"{{"jsonrpc":"2.0","id":{},"method":"tools/call","params":{{"name":"{}","arguments":{{}}}}}}"#,
            req_id, tool
        );
        stdin.write_all(req.as_bytes()).expect("write tools/call");
        stdin.write_all(b"\n").expect("write newline");
        stdin.flush().expect("flush");

        let res = read_json_line(&mut stdout);
        assert_eq!(res["jsonrpc"], "2.0");
        assert_eq!(res["id"], req_id);
        assert!(!res["result"]["isError"].as_bool().unwrap_or(false));

        let text = res["result"]["content"][0]["text"]
            .as_str()
            .expect("content text");
        let envelope: serde_json::Value = serde_json::from_str(text).expect("envelope json");
        assert_eq!(envelope["command"], *tool);
        assert!(envelope["ok"].as_bool().unwrap_or(false));
    }

    let _ = child.kill();
    let _ = child.wait();
}
