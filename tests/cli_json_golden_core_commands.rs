use std::path::{Path, PathBuf};
use std::process::Command;

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .env("AGENTPACK_MACHINE_ID", "test-machine")
        .env("HOME", home)
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

fn canonicalize_json(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut items: Vec<(String, serde_json::Value)> = map.into_iter().collect();
            items.sort_by(|a, b| a.0.cmp(&b.0));

            let mut out = serde_json::Map::new();
            for (k, v) in items {
                out.insert(k, canonicalize_json(v));
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(canonicalize_json).collect::<Vec<_>>())
        }
        other => other,
    }
}

fn normalize_json(value: serde_json::Value, tmp_prefix: &str) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if matches!(
                    k.as_str(),
                    "snapshot_id" | "event_snapshot_id" | "rolled_back_to"
                ) {
                    out.insert(k, serde_json::Value::String("<SNAPSHOT_ID>".to_string()));
                    continue;
                }
                out.insert(k, normalize_json(v, tmp_prefix));
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => serde_json::Value::Array(
            arr.into_iter()
                .map(|v| normalize_json(v, tmp_prefix))
                .collect::<Vec<_>>(),
        ),
        serde_json::Value::String(s) => {
            let mut out = s.replace('\\', "/");
            out = out.replace(tmp_prefix, "<TMP>");
            serde_json::Value::String(out)
        }
        other => other,
    }
}

fn pretty_json(value: serde_json::Value) -> String {
    let canonical = canonicalize_json(value);
    let mut out = serde_json::to_string_pretty(&canonical).expect("pretty json");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn assert_envelope_shape(v: &serde_json::Value, expected_command: &str, ok: bool) {
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["ok"], ok);
    assert_eq!(v["command"], expected_command);

    assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
    assert!(v["data"].is_object());
    assert!(v["warnings"].is_array());
    assert!(v["errors"].is_array());
}

fn assert_data_matches_golden(
    v: &serde_json::Value,
    golden_path: &str,
    tmp_prefix: &str,
) -> anyhow::Result<()> {
    let normalized = normalize_json(v["data"].clone(), tmp_prefix);
    let actual = pretty_json(normalized);

    let expected_path = std::path::Path::new(golden_path);
    if !expected_path.exists() {
        anyhow::bail!("missing golden snapshot: {golden_path}\n\n{actual}");
    }

    let expected = std::fs::read_to_string(expected_path)
        .map_err(|e| anyhow::anyhow!("read golden {golden_path}: {e}"))?;
    assert_eq!(actual, expected);
    Ok(())
}

fn write_manifest(repo_dir: &Path, codex_home: &Path) -> anyhow::Result<()> {
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
      codex_home: '{codex_home}'
      write_agents_global: true
      write_agents_repo_root: false
      write_user_prompts: true
      write_user_skills: false
      write_repo_skills: false

modules:
  - id: instructions:base
    type: instructions
    enabled: true
    tags: ["base"]
    targets: ["codex"]
    source:
      local_path:
        path: modules/instructions/base

  - id: prompt:hello
    type: prompt
    enabled: true
    tags: ["base"]
    targets: ["codex"]
    source:
      local_path:
        path: modules/prompts/hello
"#,
        codex_home = codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest)
        .map_err(|e| anyhow::anyhow!("write manifest: {e}"))?;
    Ok(())
}

fn write_module(
    repo_dir: &Path,
    rel_dir: &str,
    filename: &str,
    content: &str,
) -> anyhow::Result<()> {
    let dir = repo_dir.join(rel_dir);
    std::fs::create_dir_all(&dir).map_err(|e| anyhow::anyhow!("create module dir: {e}"))?;
    std::fs::write(dir.join(filename), content)
        .map_err(|e| anyhow::anyhow!("write module: {e}"))?;
    Ok(())
}

fn init_workspace(tmp: &tempfile::TempDir) -> anyhow::Result<PathBuf> {
    let workspace = tmp.path().join("workspace");
    std::fs::create_dir_all(&workspace)?;

    assert!(git_in(&workspace, &["init"]).status.success());
    // Provide a stable origin for deterministic project_id derivation.
    let _ = git_in(
        &workspace,
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/example/example.git",
        ],
    );

    Ok(workspace)
}

#[test]
fn json_core_command_data_matches_golden_snapshots() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(&tmp)?;

    let tmp_prefix = home.to_string_lossy().replace('\\', "/");

    let init = agentpack_in(home, &workspace, &["init", "--yes", "--json"]);
    assert!(init.status.success());
    let init_v = parse_stdout_json(&init);
    assert_envelope_shape(&init_v, "init", true);
    assert_data_matches_golden(&init_v, "tests/golden/init_json_data.json", &tmp_prefix)?;

    let repo_dir = home.join("repo");
    let codex_home = home.join("codex_home");
    std::fs::create_dir_all(&codex_home)?;

    write_module(
        &repo_dir,
        "modules/instructions/base",
        "AGENTS.md",
        "# Base instructions\n",
    )?;
    write_module(&repo_dir, "modules/prompts/hello", "hello.md", "Hello v1\n")?;
    write_manifest(&repo_dir, &codex_home)?;

    let update = agentpack_in(home, &workspace, &["update", "--yes", "--json"]);
    assert!(update.status.success());
    let update_v = parse_stdout_json(&update);
    assert_envelope_shape(&update_v, "update", true);
    assert_data_matches_golden(&update_v, "tests/golden/update_json_data.json", &tmp_prefix)?;

    let plan = agentpack_in(home, &workspace, &["--target", "codex", "plan", "--json"]);
    assert!(plan.status.success());
    let plan_v = parse_stdout_json(&plan);
    assert_envelope_shape(&plan_v, "plan", true);
    assert_data_matches_golden(&plan_v, "tests/golden/plan_json_data.json", &tmp_prefix)?;

    let diff = agentpack_in(home, &workspace, &["--target", "codex", "diff", "--json"]);
    assert!(diff.status.success());
    let diff_v = parse_stdout_json(&diff);
    assert_envelope_shape(&diff_v, "diff", true);
    assert_data_matches_golden(&diff_v, "tests/golden/diff_json_data.json", &tmp_prefix)?;

    let preview_plan = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "preview", "--json"],
    );
    assert!(preview_plan.status.success());
    let preview_plan_v = parse_stdout_json(&preview_plan);
    assert_envelope_shape(&preview_plan_v, "preview", true);
    assert_data_matches_golden(
        &preview_plan_v,
        "tests/golden/preview_json_data.json",
        &tmp_prefix,
    )?;

    let overlay_path = agentpack_in(
        home,
        &workspace,
        &["overlay", "path", "instructions:base", "--json"],
    );
    assert!(overlay_path.status.success());
    let overlay_path_v = parse_stdout_json(&overlay_path);
    assert_envelope_shape(&overlay_path_v, "overlay.path", true);
    assert_data_matches_golden(
        &overlay_path_v,
        "tests/golden/overlay_path_json_data.json",
        &tmp_prefix,
    )?;

    let doctor = agentpack_in(home, &workspace, &["--target", "codex", "doctor", "--json"]);
    assert!(doctor.status.success());
    let doctor_v = parse_stdout_json(&doctor);
    assert_envelope_shape(&doctor_v, "doctor", true);
    assert_data_matches_golden(&doctor_v, "tests/golden/doctor_json_data.json", &tmp_prefix)?;

    let preview = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "preview", "--diff", "--json"],
    );
    assert!(preview.status.success());
    let preview_v = parse_stdout_json(&preview);
    assert_envelope_shape(&preview_v, "preview", true);
    assert_data_matches_golden(
        &preview_v,
        "tests/golden/preview_diff_json_data.json",
        &tmp_prefix,
    )?;

    let deploy1 = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "deploy", "--apply", "--yes", "--json"],
    );
    assert!(deploy1.status.success());
    let deploy1_v = parse_stdout_json(&deploy1);
    assert_envelope_shape(&deploy1_v, "deploy", true);
    let snapshot1 = deploy1_v["data"]["snapshot_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(!snapshot1.is_empty());

    write_module(&repo_dir, "modules/prompts/hello", "hello.md", "Hello v2\n")?;
    let deploy2 = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "deploy", "--apply", "--yes", "--json"],
    );
    assert!(deploy2.status.success());
    let deploy2_v = parse_stdout_json(&deploy2);
    assert_envelope_shape(&deploy2_v, "deploy", true);
    assert_data_matches_golden(
        &deploy2_v,
        "tests/golden/deploy_json_data.json",
        &tmp_prefix,
    )?;

    // Create managed drift + unmanaged extra.
    std::fs::write(codex_home.join("prompts").join("hello.md"), "local drift\n")?;
    std::fs::write(codex_home.join("prompts").join("extra.txt"), "extra\n")?;

    let evolve_propose = agentpack_in(
        home,
        &workspace,
        &[
            "--target",
            "codex",
            "evolve",
            "propose",
            "--dry-run",
            "--json",
        ],
    );
    assert!(evolve_propose.status.success());
    let evolve_propose_v = parse_stdout_json(&evolve_propose);
    assert_envelope_shape(&evolve_propose_v, "evolve.propose", true);
    assert_data_matches_golden(
        &evolve_propose_v,
        "tests/golden/evolve_propose_dry_run_json_data.json",
        &tmp_prefix,
    )?;

    std::fs::remove_file(codex_home.join("prompts").join("hello.md"))?;
    let evolve_restore = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "evolve", "restore", "--yes", "--json"],
    );
    assert!(evolve_restore.status.success());
    let evolve_restore_v = parse_stdout_json(&evolve_restore);
    assert_envelope_shape(&evolve_restore_v, "evolve.restore", true);
    assert_data_matches_golden(
        &evolve_restore_v,
        "tests/golden/evolve_restore_json_data.json",
        &tmp_prefix,
    )?;

    std::fs::write(codex_home.join("prompts").join("hello.md"), "local drift\n")?;

    let status = agentpack_in(home, &workspace, &["--target", "codex", "status", "--json"]);
    assert!(status.status.success());
    let status_v = parse_stdout_json(&status);
    assert_envelope_shape(&status_v, "status", true);
    assert_data_matches_golden(&status_v, "tests/golden/status_json_data.json", &tmp_prefix)?;

    let rollback = agentpack_in(
        home,
        &workspace,
        &[
            "--target",
            "codex",
            "rollback",
            "--to",
            snapshot1.as_str(),
            "--yes",
            "--json",
        ],
    );
    assert!(rollback.status.success());
    let rollback_v = parse_stdout_json(&rollback);
    assert_envelope_shape(&rollback_v, "rollback", true);
    assert_data_matches_golden(
        &rollback_v,
        "tests/golden/rollback_json_data.json",
        &tmp_prefix,
    )?;

    Ok(())
}
