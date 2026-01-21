mod journeys;

use std::path::Path;

use journeys::common::{TestEnv, run_json_ok, write_file};

#[test]
fn journey_j2_import_existing_assets_user_and_project() {
    let env = TestEnv::new();

    // Seed user-scope assets.
    write_file(
        &env.home().join(".codex/prompts/user-prompt.md"),
        "# user prompt\n",
    );
    write_file(
        &env.home().join(".claude/commands/user-cmd.md"),
        "---\ndescription: \"user cmd\"\n---\n\nHello\n",
    );
    write_file(
        &env.home().join(".codex/skills/user-skill/SKILL.md"),
        "---\nname: user-skill\ndescription: imported user skill\n---\n\n# user-skill\n",
    );

    // Seed project-scope assets.
    write_file(&env.workspace().join("AGENTS.md"), "# Project agents\n");
    write_file(
        &env.workspace().join(".claude/commands/proj-cmd.md"),
        "---\ndescription: \"proj cmd\"\n---\n\nHello\n",
    );
    write_file(
        &env.workspace().join(".codex/skills/proj-skill/SKILL.md"),
        "---\nname: proj-skill\ndescription: imported project skill\n---\n\n# proj-skill\n",
    );

    // init
    env.init_repo();
    let manifest_before =
        std::fs::read_to_string(env.manifest_path()).expect("read manifest (before)");

    // import (dry-run)
    let dry_run = run_json_ok(&env, &["--json", "import"]);
    assert_eq!(dry_run["data"]["applied"].as_bool(), Some(false));
    assert_eq!(dry_run["data"]["reason"].as_str(), Some("dry_run"));
    assert!(
        dry_run["data"]["summary"]["create"].as_u64().unwrap_or(0) > 0,
        "expected planned creates in dry-run import"
    );

    // Dry-run should not write to the repo.
    assert_eq!(
        std::fs::read_to_string(env.manifest_path()).expect("read manifest (after dry-run)"),
        manifest_before
    );
    assert!(!env.repo_dir().join("modules/prompts/imported").exists());
    assert!(
        !env.repo_dir()
            .join("modules/claude-commands/imported")
            .exists()
    );
    assert!(!env.repo_dir().join("modules/skills/imported").exists());

    // None of the planned destinations should exist during dry-run.
    for item in dry_run["data"]["plan"]
        .as_array()
        .expect("import plan array")
    {
        if item["op"].as_str() != Some("create") {
            continue;
        }
        let dest = item["dest_path"].as_str().expect("dest_path");
        assert!(
            !Path::new(dest).exists(),
            "dry-run created destination unexpectedly: {dest}"
        );
    }

    // import --apply
    let applied = run_json_ok(&env, &["--json", "--yes", "import", "--apply"]);
    assert_eq!(applied["data"]["applied"].as_bool(), Some(true));

    // Planned destinations should exist after apply.
    for item in applied["data"]["plan"]
        .as_array()
        .expect("import plan array")
    {
        if item["op"].as_str() != Some("create") {
            continue;
        }
        let dest = item["dest_path"].as_str().expect("dest_path");
        assert!(
            Path::new(dest).exists(),
            "missing imported destination: {dest}"
        );
    }

    let manifest_after =
        std::fs::read_to_string(env.manifest_path()).expect("read manifest (after apply)");
    assert_ne!(manifest_after, manifest_before);

    // Post-import preview/deploy should succeed (project profile required when project items exist).
    let project_id = applied["data"]["project"]["project_id"]
        .as_str()
        .expect("project_id");
    let project_profile = format!("project-{project_id}");

    run_json_ok(
        &env,
        &["--profile", &project_profile, "--json", "preview", "--diff"],
    );

    let deploy = run_json_ok(
        &env,
        &[
            "--profile",
            &project_profile,
            "--json",
            "--yes",
            "deploy",
            "--apply",
        ],
    );
    assert_eq!(deploy["data"]["applied"].as_bool(), Some(true));
}
