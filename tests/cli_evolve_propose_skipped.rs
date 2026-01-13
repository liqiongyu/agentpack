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
fn evolve_propose_reports_skipped_missing_and_multi_module_drift() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    let repo_dir = tmp.path().join("repo");
    let i1 = repo_dir.join("modules/instructions/one");
    let i2 = repo_dir.join("modules/instructions/two");
    std::fs::create_dir_all(&i1).expect("create instructions module 1");
    std::fs::create_dir_all(&i2).expect("create instructions module 2");
    std::fs::write(i1.join("AGENTS.md"), "# one\n").expect("write AGENTS 1");
    std::fs::write(i2.join("AGENTS.md"), "# two\n").expect("write AGENTS 2");

    let p1 = repo_dir.join("modules/prompt/one");
    std::fs::create_dir_all(&p1).expect("create prompt module dir");
    std::fs::write(p1.join("prompt.md"), "# prompt\n").expect("write prompt");

    // Create drift for aggregated instructions output (multi-module output).
    std::fs::write(codex_home.join("AGENTS.md"), "drifted\n").expect("write drifted AGENTS");

    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: ["instructions:one","instructions:two","prompt:one"]
    exclude_modules: []

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_agents_global: true
      write_agents_repo_root: false
      write_user_prompts: true
      write_user_skills: false
      write_repo_skills: false

modules:
  - id: "instructions:one"
    type: instructions
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/instructions/one"
  - id: "instructions:two"
    type: instructions
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/instructions/two"
  - id: "prompt:one"
    type: prompt
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/prompt/one"
"#,
        codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");

    // The prompt output is expected under codex_home/prompts/prompt.md, but we leave it missing.
    let out = agentpack_in(
        tmp.path(),
        &[
            "--target",
            "codex",
            "evolve",
            "propose",
            "--dry-run",
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "evolve.propose");
    assert_eq!(v["data"]["created"], false);
    assert_eq!(v["data"]["reason"], "no_proposeable_drift");

    let summary = &v["data"]["summary"];
    assert_eq!(summary["drifted_proposeable"], 0);
    assert_eq!(summary["skipped_missing"], 1);
    assert_eq!(summary["skipped_multi_module"], 1);

    let skipped = v["data"]["skipped"].as_array().expect("skipped array");
    assert_eq!(skipped.len(), 2);
    assert!(skipped.iter().any(|s| s["reason"] == "missing"));
    assert!(skipped.iter().any(|s| s["reason"] == "multi_module_output"));

    let missing = skipped
        .iter()
        .find(|s| s["reason"] == "missing")
        .expect("missing skipped item");
    let missing_suggestions = missing["suggestions"]
        .as_array()
        .expect("missing suggestions array");
    assert!(
        missing_suggestions
            .iter()
            .any(|s| s["action"] == "agentpack evolve restore"),
        "missing suggestions include evolve restore"
    );
    assert!(
        missing_suggestions
            .iter()
            .any(|s| s["action"] == "agentpack deploy --apply"),
        "missing suggestions include deploy --apply"
    );

    let multi = skipped
        .iter()
        .find(|s| s["reason"] == "multi_module_output")
        .expect("multi_module_output skipped item");
    let multi_suggestions = multi["suggestions"]
        .as_array()
        .expect("multi_module_output suggestions array");
    assert!(
        multi_suggestions
            .iter()
            .any(|s| s["action"] == "Add per-module markers to aggregated instructions outputs"),
        "multi_module_output suggestions include per-module markers"
    );
    assert!(
        multi_suggestions
            .iter()
            .any(|s| s["action"] == "Split aggregated outputs so each file maps to one module"),
        "multi_module_output suggestions include split outputs"
    );
}
