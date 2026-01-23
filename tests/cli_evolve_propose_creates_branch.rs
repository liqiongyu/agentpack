mod journeys;

use journeys::common::{TestEnv, git_ok, git_stdout, run_json_ok, write_file};

#[test]
fn evolve_propose_creates_branch_and_commits_overlay_updates() {
    let env = TestEnv::new();

    // Ensure the config repo is a git repo (required by evolve propose).
    let init = env
        .agentpack()
        .args(["--json", "--yes", "init", "--git"])
        .output()
        .expect("run agentpack init --git");
    assert!(
        init.status.success(),
        "init --git failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr),
    );

    let repo_dir = env.repo_dir();

    // Ensure commits work in the config repo without relying on global git config.
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

    // Keep the config repo clean before running evolve propose.
    let status = git_stdout(&repo_dir, &["status", "--porcelain"]);
    assert!(
        !status.trim().is_empty(),
        "expected config repo to have changes before committing"
    );
    git_ok(&repo_dir, &["add", "-A"]);
    git_ok(&repo_dir, &["commit", "-m", "chore(test): seed repo"]);

    let original_branch = git_stdout(&repo_dir, &["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();
    assert!(!original_branch.is_empty());

    // Deploy once so drift exists under the target root.
    let deploy = run_json_ok(
        &env,
        &["--target", "codex", "deploy", "--apply", "--json", "--yes"],
    );
    assert_eq!(deploy["ok"], true);

    let prompt_path = codex_home.join("prompts").join("prompt.md");
    assert!(prompt_path.is_file(), "expected prompt to be deployed");

    // Create proposable drift (single-module output).
    write_file(&prompt_path, "# prompt edited\n");

    // Propose drift into a global overlay on a named branch.
    let branch = "evolve/propose-test";
    let proposed = run_json_ok(
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
            "--branch",
            branch,
            "--json",
            "--yes",
        ],
    );
    assert_eq!(proposed["ok"], true);
    assert_eq!(proposed["command"], "evolve.propose");
    assert_eq!(proposed["data"]["created"], true);
    assert_eq!(proposed["data"]["branch"], branch);
    assert_eq!(proposed["data"]["scope"], "global");
    assert_eq!(proposed["data"]["committed"], true);

    // On success, evolve propose returns to the original branch.
    let head = git_stdout(&repo_dir, &["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();
    assert_eq!(head, original_branch);

    // The proposal branch exists and contains overlay updates.
    let refs = format!("refs/heads/{branch}");
    git_ok(&repo_dir, &["show-ref", "--verify", refs.as_str()]);

    let files_posix = proposed["data"]["files_posix"]
        .as_array()
        .expect("files_posix array");
    let overlay_file = files_posix
        .iter()
        .filter_map(|v| v.as_str())
        .find(|p| p.ends_with("prompt.md"))
        .expect("overlay prompt.md touched");

    let spec = format!("{branch}:{overlay_file}");
    let overlay_text = git_stdout(&repo_dir, &["show", spec.as_str()]);
    assert!(overlay_text.contains("prompt edited"));

    // Ensure the config repo working tree is clean after propose.
    let status_after = git_stdout(&repo_dir, &["status", "--porcelain"]);
    assert!(
        status_after.trim().is_empty(),
        "expected clean working tree after propose; got:\n{status_after}"
    );
}
