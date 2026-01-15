use std::path::Path;

use agentpack::config::Manifest;
use agentpack::engine::Engine;
use agentpack::paths::{AgentpackHome, RepoPaths};
use agentpack::project::ProjectContext;
use agentpack::store::Store;

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
      write_user_prompts: false
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
"#,
        codex_home = codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest)?;
    Ok(())
}

fn write_file(repo_dir: &Path, rel_dir: &str, filename: &str, content: &str) -> anyhow::Result<()> {
    let dir = repo_dir.join(rel_dir);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(filename), content)?;
    Ok(())
}

fn make_home(tmp: &tempfile::TempDir) -> anyhow::Result<AgentpackHome> {
    let root = tmp.path().join("agentpack_home");
    let state_dir = root.join("state");
    let home = AgentpackHome {
        repo_dir: root.join("repo"),
        cache_dir: root.join("cache"),
        snapshots_dir: state_dir.join("snapshots"),
        logs_dir: state_dir.join("logs"),
        state_dir,
        root,
    };
    std::fs::create_dir_all(&home.cache_dir)?;
    std::fs::create_dir_all(&home.snapshots_dir)?;
    std::fs::create_dir_all(&home.logs_dir)?;
    Ok(home)
}

#[test]
fn tui_core_can_collect_read_only_views() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let home = make_home(&tmp)?;

    let repo_dir = tmp.path().join("repo");
    std::fs::create_dir_all(&repo_dir)?;

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(&codex_home)?;

    write_manifest(&repo_dir, &codex_home)?;
    write_file(
        &repo_dir,
        "modules/instructions/base",
        "AGENTS.md",
        "# Test instructions\n",
    )?;

    let repo = RepoPaths::resolve(&home, Some(&repo_dir))?;
    let manifest = Manifest::load(&repo.manifest_path)?;
    let store = Store::new(&home);
    let project = ProjectContext::detect(tmp.path())?;

    let engine = Engine {
        home,
        repo,
        manifest,
        lockfile: None,
        store,
        project,
        machine_id: "test-machine".to_string(),
    };

    let views = agentpack::tui_core::collect_read_only_text_views(&engine, "default", "codex")?;
    assert!(views.plan.contains("Plan:"));
    assert!(!views.diff.is_empty());
    assert!(views.status.contains("No drift") || views.status.contains("Drift"));

    Ok(())
}
