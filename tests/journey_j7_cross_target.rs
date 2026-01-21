mod journeys;

use journeys::common::{TestEnv, read_text_normalized, run_json_ok, write_file};

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

    run_json_ok(&env, &["--json", "--yes", "update"]);

    let deploy_v1 = run_json_ok(
        &env,
        &["--target", "all", "--json", "--yes", "deploy", "--apply"],
    );
    assert!(deploy_v1["ok"].as_bool().unwrap_or(false));
    assert!(deploy_v1["data"]["applied"].as_bool().unwrap_or(false));
    let snapshot_v1 = deploy_v1["data"]["snapshot_id"]
        .as_str()
        .expect("snapshot_id")
        .to_string();
    assert!(!snapshot_v1.trim().is_empty());

    let codex_skill = codex_home.join("skills").join("j7-skill").join("SKILL.md");
    let claude_skill = env
        .workspace()
        .join(".claude")
        .join("skills")
        .join("j7-skill")
        .join("SKILL.md");
    assert!(codex_skill.exists(), "expected {codex_skill:?}");
    assert!(claude_skill.exists(), "expected {claude_skill:?}");
    assert!(
        codex_home
            .join("skills")
            .join(".agentpack.manifest.codex.json")
            .exists()
    );
    assert!(
        env.workspace()
            .join(".claude")
            .join("skills")
            .join(".agentpack.manifest.claude_code.json")
            .exists()
    );

    assert!(read_text_normalized(&codex_skill).contains("# j7-skill v1"));
    assert!(read_text_normalized(&claude_skill).contains("# j7-skill v1"));

    write_file(
        &skill_dir.join("SKILL.md"),
        "---\nname: j7-skill\ndescription: Journey J7 cross-target skill\n---\n\n# j7-skill v2\n",
    );

    let deploy_v2 = run_json_ok(
        &env,
        &["--target", "all", "--json", "--yes", "deploy", "--apply"],
    );
    assert!(deploy_v2["ok"].as_bool().unwrap_or(false));
    assert!(deploy_v2["data"]["applied"].as_bool().unwrap_or(false));

    assert!(read_text_normalized(&codex_skill).contains("# j7-skill v2"));
    assert!(read_text_normalized(&claude_skill).contains("# j7-skill v2"));

    let rollback = run_json_ok(
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
    );
    assert!(rollback["ok"].as_bool().unwrap_or(false));
    assert_eq!(
        rollback["data"]["rolled_back_to"].as_str(),
        Some(snapshot_v1.as_str())
    );

    assert!(read_text_normalized(&codex_skill).contains("# j7-skill v1"));
    assert!(read_text_normalized(&claude_skill).contains("# j7-skill v1"));
}
