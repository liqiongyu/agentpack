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

#[test]
fn journey_j3_adopt_update_refuse_then_adopt_then_managed_update() {
    let env = TestEnv::new();

    env.init_repo_with_base_modules();
    run_ok(&env, &["--json", "--yes", "update"]);

    // Seed an unmanaged existing file at a desired output path.
    let prompt_path = env.home().join(".codex/prompts/draftpr.md");
    write_file(&prompt_path, "# unmanaged existing prompt\n");

    // deploy --apply without --adopt should fail with the stable error code.
    let refused = parse_json(&run_fail(
        &env,
        &["--target", "codex", "--json", "--yes", "deploy", "--apply"],
    ));
    assert_eq!(refused["ok"].as_bool(), Some(false));
    assert_eq!(
        refused["errors"][0]["code"].as_str(),
        Some("E_ADOPT_CONFIRM_REQUIRED")
    );
    assert_eq!(
        refused["errors"][0]["details"]["flag"].as_str(),
        Some("--adopt")
    );
    assert!(
        refused["errors"][0]["details"]["adopt_updates"]
            .as_u64()
            .unwrap_or(0)
            >= 1
    );
    let prompt_path_str = prompt_path.to_string_lossy().to_string();
    let sample_paths = refused["errors"][0]["details"]["sample_paths"]
        .as_array()
        .expect("sample_paths array");
    assert!(
        sample_paths
            .iter()
            .any(|p| p.as_str() == Some(prompt_path_str.as_str())),
        "expected sample_paths to include {prompt_path_str}; got {sample_paths:?}"
    );

    // deploy --apply with --adopt should succeed.
    let adopted = parse_json(&run_ok(
        &env,
        &[
            "--target", "codex", "--json", "--yes", "deploy", "--apply", "--adopt",
        ],
    ));
    assert_eq!(adopted["ok"].as_bool(), Some(true));
    assert_eq!(adopted["data"]["applied"].as_bool(), Some(true));

    // Mutate the upstream module so a follow-up update is required.
    let prompt_src = env.repo_dir().join("modules/prompts/draftpr.md");
    let original = std::fs::read_to_string(&prompt_src).expect("read prompt src");
    std::fs::write(
        &prompt_src,
        format!("{original}\n\n<!-- journey-j3-managed-update -->\n"),
    )
    .expect("write prompt src");

    // The follow-up update should be managed_update and should not require --adopt.
    let updated = parse_json(&run_ok(
        &env,
        &["--target", "codex", "--json", "--yes", "deploy", "--apply"],
    ));
    assert_eq!(updated["ok"].as_bool(), Some(true));
    assert_eq!(updated["data"]["applied"].as_bool(), Some(true));
    let changes = updated["data"]["changes"]
        .as_array()
        .expect("deploy changes array");
    let prompt_change = changes
        .iter()
        .find(|c| c["path"].as_str() == Some(prompt_path_str.as_str()))
        .expect("prompt path change");
    assert_eq!(prompt_change["op"].as_str(), Some("update"));
    assert_eq!(
        prompt_change["update_kind"].as_str(),
        Some("managed_update")
    );
    assert!(
        !changes
            .iter()
            .any(|c| c["update_kind"].as_str() == Some("adopt_update")),
        "expected no adopt_update changes after adoption; got {changes:?}"
    );
}
