use std::path::Path;
use std::process::Command;

use agentpack::config::{LocalPathSource, Manifest, Module, ModuleType, Source};
use agentpack::ids::module_fs_key;

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .env("EDITOR", "")
        .output()
        .expect("run agentpack")
}

#[test]
fn overlay_edit_kind_patch_creates_patch_overlay_skeleton() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let home = tmp.path();

    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir)?;

    let init = agentpack_in(home, &project_dir, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");

    let module_root = repo_dir.join("modules").join("skill_test");
    std::fs::create_dir_all(&module_root)?;
    std::fs::write(module_root.join("SKILL.md"), "line1\nline2\n")?;

    let manifest_path = repo_dir.join("agentpack.yaml");
    let mut manifest = Manifest::load(&manifest_path)?;
    manifest.modules.push(Module {
        id: "skill_test".to_string(),
        module_type: ModuleType::Skill,
        enabled: true,
        tags: Vec::new(),
        targets: Vec::new(),
        source: Source {
            local_path: Some(LocalPathSource {
                path: "modules/skill_test".to_string(),
            }),
            git: None,
        },
        metadata: Default::default(),
    });
    manifest.save(&manifest_path)?;

    let edit = agentpack_in(
        home,
        &project_dir,
        &[
            "overlay",
            "edit",
            "skill_test",
            "--scope",
            "global",
            "--kind",
            "patch",
        ],
    );
    assert!(edit.status.success(), "overlay edit should succeed");

    let overlay_dir = repo_dir.join("overlays").join(module_fs_key("skill_test"));

    assert!(overlay_dir.exists());
    assert!(overlay_dir.join(".agentpack/baseline.json").exists());
    assert!(overlay_dir.join(".agentpack/module_id").exists());
    assert!(overlay_dir.join(".agentpack/patches").is_dir());

    let raw = std::fs::read_to_string(overlay_dir.join(".agentpack/overlay.json"))?;
    let meta: serde_json::Value = serde_json::from_str(&raw)?;
    assert_eq!(meta["overlay_kind"], "patch");

    // Patch overlays should not copy upstream files into the overlay root.
    assert!(!overlay_dir.join("SKILL.md").exists());

    Ok(())
}
