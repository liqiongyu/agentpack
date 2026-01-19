mod journeys;

use std::path::Path;
use std::process::Output;

use journeys::common::TestEnv;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(path, contents).expect("write file");
}

fn run_out(env: &TestEnv, args: &[&str]) -> Output {
    env.agentpack().args(args).output().expect("run agentpack")
}

fn run_ok(env: &TestEnv, args: &[&str]) -> Output {
    let out = run_out(env, args);
    assert!(
        out.status.success(),
        "command failed: agentpack {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    out
}

fn parse_json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).expect("parse json stdout")
}

fn read_text_normalized(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .replace("\r\n", "\n")
}

#[test]
fn journey_j7_cross_target_deploy_and_rollback_are_consistent() {
    let env = TestEnv::new();
    env.init_repo();

    let repo_dir = env.repo_dir();
    let codex_home = env.home().join(".codex");

    let skill_dir = repo_dir.join("modules/skills/j7-skill");
    write_file(
        &skill_dir.join("SKILL.md"),
        "---\nname: j7-skill\ndescription: Journey J7 cross-target skill\n---\n\n# j7-skill v1\n",
    );

    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: ["j7"]

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{codex_home}'
      write_agents_global: false
      write_agents_repo_root: false
      write_user_prompts: false
      write_user_skills: true
      write_repo_skills: false
  claude_code:
    mode: files
    scope: project
    options:
      write_repo_commands: false
      write_user_commands: false
      write_repo_skills: true
      write_user_skills: false

modules:
  - id: skill:j7-skill
    type: skill
    tags: ["j7"]
    targets: ["codex", "claude_code"]
    source:
      local_path:
        path: "modules/skills/j7-skill"
"#,
        codex_home = codex_home.display(),
    );
    write_file(&env.manifest_path(), &manifest);

    run_ok(&env, &["--json", "--yes", "update"]);

    let deploy_v1 = parse_json(&run_ok(
        &env,
        &["--target", "all", "--json", "--yes", "deploy", "--apply"],
    ));
    assert!(deploy_v1["ok"].as_bool().unwrap_or(false));
    assert!(deploy_v1["data"]["applied"].as_bool().unwrap_or(false));
    let snapshot_v1 = deploy_v1["data"]["snapshot_id"]
        .as_str()
        .expect("snapshot_id")
        .to_string();

    let codex_skill = codex_home.join("skills/j7-skill/SKILL.md");
    let claude_skill = env.workspace().join(".claude/skills/j7-skill/SKILL.md");
    assert!(codex_skill.exists(), "expected {codex_skill:?}");
    assert!(claude_skill.exists(), "expected {claude_skill:?}");
    assert!(
        codex_home
            .join("skills/.agentpack.manifest.codex.json")
            .exists()
    );
    assert!(
        env.workspace()
            .join(".claude/skills/.agentpack.manifest.claude_code.json")
            .exists()
    );

    assert!(read_text_normalized(&codex_skill).contains("# j7-skill v1"));
    assert!(read_text_normalized(&claude_skill).contains("# j7-skill v1"));

    write_file(
        &skill_dir.join("SKILL.md"),
        "---\nname: j7-skill\ndescription: Journey J7 cross-target skill\n---\n\n# j7-skill v2\n",
    );

    let deploy_v2 = parse_json(&run_ok(
        &env,
        &["--target", "all", "--json", "--yes", "deploy", "--apply"],
    ));
    assert!(deploy_v2["ok"].as_bool().unwrap_or(false));
    assert!(deploy_v2["data"]["applied"].as_bool().unwrap_or(false));

    assert!(read_text_normalized(&codex_skill).contains("# j7-skill v2"));
    assert!(read_text_normalized(&claude_skill).contains("# j7-skill v2"));

    let rollback = parse_json(&run_ok(
        &env,
        &[
            "--target",
            "all",
            "--json",
            "--yes",
            "rollback",
            "--to",
            snapshot_v1.as_str(),
        ],
    ));
    assert!(rollback["ok"].as_bool().unwrap_or(false));
    assert_eq!(
        rollback["data"]["rolled_back_to"].as_str(),
        Some(snapshot_v1.as_str())
    );

    assert!(read_text_normalized(&codex_skill).contains("# j7-skill v1"));
    assert!(read_text_normalized(&claude_skill).contains("# j7-skill v1"));
}
