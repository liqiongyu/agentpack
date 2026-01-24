mod journeys;

use std::path::PathBuf;

use journeys::common::{TestEnv, git_ok, run_json_fail, run_json_ok, write_file};

#[test]
fn journey_j4_overlay_sparse_materialize_rebase_conflict_then_deploy() {
    let env = TestEnv::new();
    let module_id = "instructions:base";

    env.init_repo_with_base_modules();
    run_json_ok(&env, &["--json", "--yes", "update"]);

    // Make the upstream file deterministic (this is what the overlay will be based on).
    let upstream_agents = env
        .repo_dir()
        .join("modules")
        .join("instructions")
        .join("base")
        .join("AGENTS.md");
    let base_text = "line: BASE\n";
    write_file(&upstream_agents, base_text);

    // Local-path rebase requires a git merge base; initialize and commit the repo state.
    let repo_dir = env.repo_dir();
    git_ok(&repo_dir, &["init"]);
    git_ok(&repo_dir, &["config", "user.email", "test@example.com"]);
    git_ok(&repo_dir, &["config", "user.name", "Test User"]);
    git_ok(&repo_dir, &["add", "."]);
    git_ok(&repo_dir, &["commit", "-m", "baseline"]);

    // overlay edit --sparse should not copy upstream files into the overlay.
    let edit_sparse = run_json_ok(
        &env,
        &["--json", "--yes", "overlay", "edit", module_id, "--sparse"],
    );
    let overlay_dir = PathBuf::from(
        edit_sparse["data"]["overlay_dir"]
            .as_str()
            .expect("overlay_dir"),
    );
    let overlay_agents = overlay_dir.join("AGENTS.md");
    assert!(!overlay_agents.exists());

    // overlay edit --materialize should populate upstream files missing-only.
    let edit_materialize = run_json_ok(
        &env,
        &[
            "--json",
            "--yes",
            "overlay",
            "edit",
            module_id,
            "--materialize",
        ],
    );
    assert_eq!(
        edit_materialize["data"]["materialized"].as_bool(),
        Some(true)
    );
    assert!(overlay_agents.exists());
    assert_eq!(
        std::fs::read_to_string(&overlay_agents).expect("read overlay agents"),
        base_text
    );

    // Edit the overlay ("ours") and then update upstream ("theirs") in a conflicting way.
    let ours_text = "line: OURS\n";
    let theirs_text = "line: THEIRS\n";
    write_file(&overlay_agents, ours_text);
    write_file(&upstream_agents, theirs_text);
    git_ok(&repo_dir, &["add", "modules/instructions/base/AGENTS.md"]);
    git_ok(&repo_dir, &["commit", "-m", "upstream update"]);

    // overlay rebase should surface the stable conflict error code and write conflict markers.
    let rebase = run_json_fail(&env, &["--json", "--yes", "overlay", "rebase", module_id]);
    assert_eq!(rebase["ok"].as_bool(), Some(false));
    assert_eq!(
        rebase["errors"][0]["code"].as_str(),
        Some("E_OVERLAY_REBASE_CONFLICT")
    );
    assert_eq!(
        rebase["errors"][0]["details"]["reason_code"].as_str(),
        Some("overlay_rebase_conflict")
    );
    assert_eq!(
        rebase["errors"][0]["details"]["next_actions"],
        serde_json::json!(["resolve_overlay_conflicts", "retry_overlay_rebase"])
    );
    let conflicts = rebase["errors"][0]["details"]["conflicts"]
        .as_array()
        .expect("conflicts array");
    assert!(
        conflicts.iter().any(|c| c.as_str() == Some("AGENTS.md")),
        "expected conflicts to include AGENTS.md; got {conflicts:?}"
    );
    let conflict_marked = std::fs::read_to_string(&overlay_agents).expect("read conflict overlay");
    assert!(
        conflict_marked.contains("<<<<<<<"),
        "expected conflict markers in overlay file; got:\n{conflict_marked}"
    );

    // Resolve conflict manually and ensure deploy uses the overlay-composed content.
    let resolved_text = "line: RESOLVED\n";
    write_file(&overlay_agents, resolved_text);

    let deploy = run_json_ok(
        &env,
        &["--target", "codex", "--json", "--yes", "deploy", "--apply"],
    );
    assert_eq!(deploy["ok"].as_bool(), Some(true));
    assert_eq!(deploy["data"]["applied"].as_bool(), Some(true));

    let deployed_agents = env.home().join(".codex").join("AGENTS.md");
    assert_eq!(
        std::fs::read_to_string(&deployed_agents).expect("read deployed AGENTS.md"),
        resolved_text
    );
}
