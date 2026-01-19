mod journeys;

use std::path::{Path, PathBuf};
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

fn run_fail(env: &TestEnv, args: &[&str]) -> Output {
    let out = run_out(env, args);
    assert!(
        !out.status.success(),
        "command unexpectedly succeeded: agentpack {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    out
}

fn parse_json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).expect("parse json stdout")
}

fn git_out(dir: &Path, args: &[&str]) -> Output {
    std::process::Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git")
}

fn git_ok(dir: &Path, args: &[&str]) {
    let out = git_out(dir, args);
    assert!(
        out.status.success(),
        "git command failed: git {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

#[test]
fn journey_j5_patch_overlay_generate_rebase_conflict_then_deploy() {
    let env = TestEnv::new();
    let module_id = "instructions:base";

    env.init_repo_with_base_modules();
    run_ok(&env, &["--json", "--yes", "update"]);

    let upstream_agents = env
        .repo_dir()
        .join("modules")
        .join("instructions")
        .join("base")
        .join("AGENTS.md");

    let base_text = "line1\nline2\nline3\n";
    write_file(&upstream_agents, base_text);

    // Local-path rebase requires a git merge base; initialize and commit the repo state.
    let repo_dir = env.repo_dir();
    git_ok(&repo_dir, &["init"]);
    git_ok(&repo_dir, &["config", "user.email", "test@example.com"]);
    git_ok(&repo_dir, &["config", "user.name", "Test User"]);
    git_ok(&repo_dir, &["add", "."]);
    git_ok(&repo_dir, &["commit", "-m", "baseline"]);

    // Create a patch overlay via CLI and add a patch file.
    let edit_patch = parse_json(&run_ok(
        &env,
        &[
            "--json", "--yes", "overlay", "edit", module_id, "--kind", "patch",
        ],
    ));
    let overlay_dir = PathBuf::from(
        edit_patch["data"]["overlay_dir"]
            .as_str()
            .expect("overlay_dir"),
    );
    assert!(
        !overlay_dir.join("AGENTS.md").exists(),
        "patch overlays should not copy upstream files"
    );

    let overlay_meta = overlay_dir.join(".agentpack/overlay.json");
    assert!(
        overlay_meta.exists(),
        "expected overlay metadata at {}",
        overlay_meta.display()
    );
    let patch_dir = overlay_dir.join(".agentpack/patches");
    assert!(
        patch_dir.is_dir(),
        "expected patch dir at {}",
        patch_dir.display()
    );

    let patch_file = patch_dir.join("AGENTS.md.patch");
    std::fs::write(
        &patch_file,
        "--- a/AGENTS.md\n+++ b/AGENTS.md\n@@ -1,3 +1,3 @@\n line1\n-line2\n+line2-ours\n line3\n",
    )
    .expect("write patch file");

    let deploy_patched = parse_json(&run_ok(
        &env,
        &["--target", "codex", "--json", "--yes", "deploy", "--apply"],
    ));
    assert_eq!(deploy_patched["ok"].as_bool(), Some(true));
    assert_eq!(deploy_patched["data"]["applied"].as_bool(), Some(true));

    let deployed_agents = env.home().join(".codex").join("AGENTS.md");
    assert_eq!(
        std::fs::read_to_string(&deployed_agents).expect("read deployed AGENTS.md"),
        "line1\nline2-ours\nline3\n"
    );

    // Upstream edit conflicts with the patch; overlay rebase should fail with stable error code
    // and write conflict artifacts.
    let theirs_text = "line1\nline2-theirs\nline3\n";
    write_file(&upstream_agents, theirs_text);
    git_ok(&repo_dir, &["add", "modules/instructions/base/AGENTS.md"]);
    git_ok(&repo_dir, &["commit", "-m", "upstream change"]);

    let rebase = parse_json(&run_fail(
        &env,
        &["--json", "--yes", "overlay", "rebase", module_id],
    ));
    assert_eq!(rebase["ok"].as_bool(), Some(false));
    assert_eq!(
        rebase["errors"][0]["code"].as_str(),
        Some("E_OVERLAY_REBASE_CONFLICT")
    );
    let conflicts = rebase["errors"][0]["details"]["conflicts"]
        .as_array()
        .expect("conflicts array");
    assert!(
        conflicts.iter().any(|c| c.as_str() == Some("AGENTS.md")),
        "expected conflicts to include AGENTS.md; got {conflicts:?}"
    );

    let conflict_artifact = overlay_dir.join(".agentpack/conflicts/AGENTS.md");
    assert!(
        conflict_artifact.exists(),
        "expected conflict artifact at {}",
        conflict_artifact.display()
    );
    let conflict_text =
        std::fs::read_to_string(&conflict_artifact).expect("read conflict artifact");
    assert!(
        conflict_text.contains("<<<<<<<"),
        "expected conflict markers in conflict artifact; got:\n{conflict_text}"
    );

    // Resolve by rewriting the patch against the updated upstream and redeploy.
    let resolved_text = "line1\nline2-resolved\nline3\n";
    std::fs::write(
        &patch_file,
        "--- a/AGENTS.md\n+++ b/AGENTS.md\n@@ -1,3 +1,3 @@\n line1\n-line2-theirs\n+line2-resolved\n line3\n",
    )
    .expect("write resolved patch file");

    let deploy_resolved = parse_json(&run_ok(
        &env,
        &["--target", "codex", "--json", "--yes", "deploy", "--apply"],
    ));
    assert_eq!(deploy_resolved["ok"].as_bool(), Some(true));
    assert_eq!(deploy_resolved["data"]["applied"].as_bool(), Some(true));

    assert_eq!(
        std::fs::read_to_string(&deployed_agents).expect("read deployed AGENTS.md"),
        resolved_text
    );
}
