#[cfg(not(windows))]
use std::path::Path;

#[cfg(not(windows))]
use std::process::Command;

#[test]
#[cfg(not(windows))]
fn demo_5min_script_succeeds() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/demo_5min.sh");
    assert!(script.exists(), "missing script: {}", script.display());

    let bin = env!("CARGO_BIN_EXE_agentpack");
    let out = Command::new("bash")
        .arg(script)
        .env("AGENTPACK_BIN", bin)
        .output()
        .expect("run demo_5min.sh");

    assert!(
        out.status.success(),
        "demo_5min.sh failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
