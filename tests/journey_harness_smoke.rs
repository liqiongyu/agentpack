mod journeys;

use assert_cmd::prelude::*;
use journeys::common::TestEnv;

#[test]
fn journey_harness_smoke_init_and_plan() {
    let env = TestEnv::new();
    env.init_repo_with_base_modules();

    assert!(env.home().exists());
    assert!(env.agentpack_home().exists());
    assert!(env.workspace().join(".git").exists());
    assert!(env.git(&["status", "--porcelain"]).status.success());

    env.agentpack().args(["--json", "plan"]).assert().success();
}
