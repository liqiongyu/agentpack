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

fn git_in(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

#[test]
fn doctor_warns_when_manifest_is_not_ignored_and_fix_is_idempotent() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let init = agentpack_in(tmp.path(), &["init"]);
    assert!(init.status.success());

    // Create a git repo that contains the target root.
    let repo_root = tmp.path().join("workspace");
    std::fs::create_dir_all(&repo_root).expect("create repo_root");
    let out = git_in(&repo_root, &["init"]);
    assert!(out.status.success(), "git init failed");

    let codex_home = repo_root.join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    // Point the codex target root into the git repo, and limit doctor to `--target codex`
    // so tests don't depend on local ~/.codex or project roots.
    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_agents_global: true
      write_user_prompts: false
      write_user_skills: false

modules: []
"#,
        codex_home.display()
    );
    let manifest_path = tmp.path().join("repo").join("agentpack.yaml");
    std::fs::write(&manifest_path, manifest).expect("write manifest");

    let first = agentpack_in(tmp.path(), &["--target", "codex", "doctor", "--json"]);
    assert!(first.status.success());
    let first_json = parse_stdout_json(&first);
    let warnings = first_json["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| w
            .as_str()
            .unwrap_or_default()
            .contains(".agentpack.manifest*.json")),
        "expected gitignore warning"
    );

    // In JSON mode, --fix requires --yes.
    let fix_no_yes = agentpack_in(
        tmp.path(),
        &["--target", "codex", "doctor", "--fix", "--json"],
    );
    assert!(!fix_no_yes.status.success());
    let fix_no_yes_json = parse_stdout_json(&fix_no_yes);
    assert_eq!(fix_no_yes_json["errors"][0]["code"], "E_CONFIRM_REQUIRED");

    let gitignore_path = repo_root.join(".gitignore");
    if gitignore_path.exists() {
        let before = std::fs::read_to_string(&gitignore_path).expect("read .gitignore");
        assert!(
            !before.contains(".agentpack.manifest*.json"),
            "doctor --fix without --yes must not write"
        );
    }

    let fix_yes = agentpack_in(
        tmp.path(),
        &["--target", "codex", "doctor", "--fix", "--json", "--yes"],
    );
    assert!(fix_yes.status.success());

    let after = std::fs::read_to_string(&gitignore_path).expect("read .gitignore");
    let occurrences = after
        .lines()
        .filter(|l| l.trim() == ".agentpack.manifest*.json")
        .count();
    assert_eq!(occurrences, 1);

    // After fix, the warning should go away.
    let second = agentpack_in(tmp.path(), &["--target", "codex", "doctor", "--json"]);
    assert!(second.status.success());
    let second_json = parse_stdout_json(&second);
    let warnings = second_json["warnings"].as_array().expect("warnings array");
    assert!(
        !warnings.iter().any(|w| w
            .as_str()
            .unwrap_or_default()
            .contains(".agentpack.manifest*.json")),
        "expected warning to be resolved after doctor --fix"
    );

    // Idempotent on repeated runs.
    let fix_yes_again = agentpack_in(
        tmp.path(),
        &["--target", "codex", "doctor", "--fix", "--json", "--yes"],
    );
    assert!(fix_yes_again.status.success());
    let after_again = std::fs::read_to_string(&gitignore_path).expect("read .gitignore");
    let occurrences = after_again
        .lines()
        .filter(|l| l.trim() == ".agentpack.manifest*.json")
        .count();
    assert_eq!(occurrences, 1);
}
