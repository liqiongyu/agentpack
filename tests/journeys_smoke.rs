mod journeys;

use journeys::common::{TestEnv, run_ok};

#[test]
fn journeys_smoke_init_and_plan() {
    let env = TestEnv::new();
    env.init_repo_with_base_modules();

    assert!(env.home().exists());
    assert!(env.agentpack_home().exists());
    assert!(env.workspace().join(".git").exists());
    assert!(env.git(&["status", "--porcelain"]).status.success());

    run_ok(&env, &["--json", "plan"]);
}
