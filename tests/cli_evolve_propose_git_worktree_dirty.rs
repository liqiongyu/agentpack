mod journeys;

use journeys::common::{
    TestEnv, git_ok, git_stdout, run_json_fail, run_json_ok, run_ok, write_file,
};

#[test]
fn evolve_propose_returns_stable_error_code_when_git_worktree_dirty() {
    let env = TestEnv::new();

    run_ok(&env, &["--json", "--yes", "init", "--git"]);

    let repo_dir = env.repo_dir();
    git_ok(&repo_dir, &["config", "user.email", "test@example.com"]);
    git_ok(&repo_dir, &["config", "user.name", "Test User"]);

    let codex_home = env.home().join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    let module_dir = repo_dir.join("modules/prompt/one");
    std::fs::create_dir_all(&module_dir).expect("create prompt module dir");
    write_file(&module_dir.join("prompt.md"), "# prompt\n");

    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: ["prompt:one"]
    exclude_modules: []

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_agents_global: false
      write_agents_repo_root: false
      write_user_prompts: true
      write_user_skills: false
      write_repo_skills: false

modules:
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
    write_file(&repo_dir.join("agentpack.yaml"), &manifest);

    git_ok(&repo_dir, &["add", "-A"]);
    git_ok(&repo_dir, &["commit", "-m", "chore(test): seed repo"]);

    let deploy = run_json_ok(
        &env,
        &["--target", "codex", "deploy", "--apply", "--json", "--yes"],
    );
    assert_eq!(deploy["ok"], true);

    let prompt_path = codex_home.join("prompts").join("prompt.md");
    assert!(prompt_path.is_file(), "expected prompt to be deployed");
    write_file(&prompt_path, "# prompt edited\n");

    write_file(&repo_dir.join("dirty.txt"), "dirty\n");
    let status_dirty = git_stdout(&repo_dir, &["status", "--porcelain"]);
    assert!(!status_dirty.trim().is_empty(), "expected dirty repo");

    let v = run_json_fail(
        &env,
        &[
            "--target",
            "codex",
            "evolve",
            "propose",
            "--scope",
            "global",
            "--module-id",
            "prompt:one",
            "--json",
            "--yes",
        ],
    );
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_GIT_WORKTREE_DIRTY");
    assert_eq!(v["errors"][0]["details"]["command"], "evolve propose");
    assert!(v["errors"][0]["details"]["repo"].is_string());
    assert!(v["errors"][0]["details"]["repo_posix"].is_string());
    assert!(v["errors"][0]["details"]["hint"].is_string());
}
