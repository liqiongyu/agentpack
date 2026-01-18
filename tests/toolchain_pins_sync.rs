fn read_to_string(path: &str) -> anyhow::Result<String> {
    std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("read {path}: {e}"))
}

fn extract_toml_string_value(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start();
        let rest = rest.strip_prefix('=')?.trim_start();
        let rest = rest.strip_prefix('"')?;
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    None
}

fn extract_ci_toolchain_versions(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("toolchain:") else {
            continue;
        };
        let value = rest.trim().trim_matches('"').to_string();
        if !value.is_empty() {
            out.push(value);
        }
    }
    out
}

#[test]
fn rust_toolchain_pins_are_in_sync() -> anyhow::Result<()> {
    let cargo_toml = read_to_string("Cargo.toml")?;
    let rust_toolchain = read_to_string("rust-toolchain.toml")?;
    let ci = read_to_string(".github/workflows/ci.yml")?;

    let cargo_rust_version = extract_toml_string_value(&cargo_toml, "rust-version")
        .ok_or_else(|| anyhow::anyhow!("missing rust-version in Cargo.toml"))?;
    let rust_toolchain_channel = extract_toml_string_value(&rust_toolchain, "channel")
        .ok_or_else(|| anyhow::anyhow!("missing channel in rust-toolchain.toml"))?;
    let ci_toolchains = extract_ci_toolchain_versions(&ci);
    if ci_toolchains.is_empty() {
        anyhow::bail!("missing toolchain pins in .github/workflows/ci.yml");
    }

    let expected = cargo_rust_version;

    let mut mismatches = Vec::new();
    if rust_toolchain_channel != expected {
        mismatches.push(format!(
            "rust-toolchain.toml channel={rust_toolchain_channel} (expected {expected})"
        ));
    }

    let unique_ci: std::collections::BTreeSet<String> = ci_toolchains.into_iter().collect();
    for v in unique_ci {
        if v != expected {
            mismatches.push(format!(
                ".github/workflows/ci.yml toolchain={v} (expected {expected})"
            ));
        }
    }

    if mismatches.is_empty() {
        return Ok(());
    }

    anyhow::bail!(
        "Rust toolchain pins are out of sync. Update all of:\n- Cargo.toml rust-version\n- rust-toolchain.toml channel\n- .github/workflows/ci.yml toolchain\n\nMismatches:\n- {}",
        mismatches.join("\n- ")
    );
}
