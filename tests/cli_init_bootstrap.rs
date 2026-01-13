use std::process::Command;

#[test]
fn init_bootstrap_installs_operator_assets_into_temp_home() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = env!("CARGO_BIN_EXE_agentpack");

    let out = Command::new(bin)
        .args(["init", "--bootstrap"])
        .env("AGENTPACK_HOME", tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("run agentpack");
    assert!(out.status.success());

    let home = tmp.path();
    let repo = home.join("repo");

    assert!(
        home.join(".codex/skills/agentpack-operator/SKILL.md")
            .exists()
    );
    assert!(home.join(".claude/commands/ap-plan.md").exists());

    assert!(
        repo.join(".codex/skills/agentpack-operator/SKILL.md")
            .exists()
    );
    assert!(repo.join(".claude/commands/ap-plan.md").exists());
}
