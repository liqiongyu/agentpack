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

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

#[test]
fn overlay_path_is_filesystem_safe_for_colon_module_ids() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let init = agentpack_in(tmp.path(), &["init"]);
    assert!(init.status.success());

    let out = agentpack_in(
        tmp.path(),
        &[
            "overlay",
            "path",
            "instructions:base",
            "--scope",
            "global",
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    let overlay_dir = v["data"]["overlay_dir"].as_str().expect("overlay_dir");
    let name = std::path::Path::new(overlay_dir)
        .file_name()
        .expect("file name")
        .to_string_lossy();
    assert!(name.starts_with("instructions_base--"));
    assert!(!name.contains(':'));
}
