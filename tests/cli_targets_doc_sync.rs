use std::collections::BTreeSet;
use std::process::Command;

use anyhow::Context;

fn run_agentpack(args: &[&str]) -> std::process::Output {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .args(args)
        .env("AGENTPACK_HOME", tmp.path())
        .output()
        .expect("run agentpack")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn parse_built_in_targets_md(path: &str, marker_line: &str) -> anyhow::Result<BTreeSet<String>> {
    let text = std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("read {path}: {e}"))?;

    let mut out = BTreeSet::new();
    let mut in_list = false;
    for line in text.lines() {
        let line = line.trim();
        if !in_list {
            if line == marker_line {
                in_list = true;
            }
            continue;
        }

        if line.is_empty() {
            break;
        }

        let Some(rest) = line.strip_prefix("- `") else {
            continue;
        };
        let Some(end) = rest.find('`') else {
            continue;
        };
        out.insert(rest[..end].to_string());
    }

    if out.is_empty() {
        anyhow::bail!("failed to parse built-in targets list from {path}");
    }

    Ok(out)
}

#[test]
fn targets_docs_cover_compiled_targets() -> anyhow::Result<()> {
    let output = run_agentpack(&["help", "--json"]);
    assert!(output.status.success());
    let v = parse_stdout_json(&output);

    let compiled_targets = v["data"]["targets"]
        .as_array()
        .context("help --json data.targets must be an array")?;
    let compiled: BTreeSet<String> = compiled_targets
        .iter()
        .filter_map(|t| t.as_str().map(|s| s.to_string()))
        .collect();
    assert!(
        !compiled.is_empty(),
        "help --json targets should not be empty"
    );

    let en = parse_built_in_targets_md("docs/reference/targets.md", "Built-in targets:")?;
    let zh = parse_built_in_targets_md("docs/zh-CN/reference/targets.md", "目前内置 targets：")?;

    let missing_in_en: Vec<String> = compiled.difference(&en).cloned().collect();
    let missing_in_zh: Vec<String> = compiled.difference(&zh).cloned().collect();
    if !missing_in_en.is_empty() || !missing_in_zh.is_empty() {
        anyhow::bail!(
            "TARGETS docs are out of sync with compiled targets.\n\nMissing in docs/reference/targets.md:\n  {}\n\nMissing in docs/zh-CN/reference/targets.md:\n  {}\n",
            if missing_in_en.is_empty() {
                "(none)".to_string()
            } else {
                missing_in_en.join("\n  ")
            },
            if missing_in_zh.is_empty() {
                "(none)".to_string()
            } else {
                missing_in_zh.join("\n  ")
            }
        );
    }

    let en_only: Vec<String> = en.difference(&zh).cloned().collect();
    let zh_only: Vec<String> = zh.difference(&en).cloned().collect();
    if !en_only.is_empty() || !zh_only.is_empty() {
        anyhow::bail!(
            "TARGETS docs lists differ between languages.\n\nOnly in docs/reference/targets.md:\n  {}\n\nOnly in docs/zh-CN/reference/targets.md:\n  {}\n",
            if en_only.is_empty() {
                "(none)".to_string()
            } else {
                en_only.join("\n  ")
            },
            if zh_only.is_empty() {
                "(none)".to_string()
            } else {
                zh_only.join("\n  ")
            }
        );
    }

    Ok(())
}
