use std::path::Path;
use std::process::Command;

#[test]
#[cfg(not(windows))]
fn demo_5min_script_succeeds_on_unix() {
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

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("agentpack doctor --json"));
    assert!(stderr.contains("agentpack preview --diff --json"));
}

#[test]
#[cfg(windows)]
fn demo_5min_script_succeeds_on_windows() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/demo_5min.ps1");
    assert!(script.exists(), "missing script: {}", script.display());

    let bin = env!("CARGO_BIN_EXE_agentpack");

    let out = Command::new("pwsh")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&script)
        .env("AGENTPACK_BIN", bin)
        .output()
        .or_else(|_| {
            Command::new("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
                .arg(&script)
                .env("AGENTPACK_BIN", bin)
                .output()
        })
        .expect("run demo_5min.ps1");

    assert!(
        out.status.success(),
        "demo_5min.ps1 failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("agentpack doctor --json"));
    assert!(stderr.contains("agentpack preview --diff --json"));
}
