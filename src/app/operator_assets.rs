use anyhow::Context as _;

pub(crate) fn extract_agentpack_version(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some((_, rest)) = line.split_once("agentpack_version:") {
            let mut value = rest.trim();
            value = value.trim_end_matches("-->");
            value = value.trim();
            value = value.trim_matches(|c| c == '"' || c == '\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

pub(crate) fn check_operator_file(
    path: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
    record_next_action: &mut impl FnMut(&str),
) -> anyhow::Result<()> {
    if !path.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            path.display()
        ));
        record_next_action(suggested);
        return Ok(());
    }

    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read operator asset {}", path.display()))?;
    let Some(have) = extract_agentpack_version(&text) else {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        record_next_action(suggested);
        return Ok(());
    };

    if have != current {
        warnings.push(format!(
            "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
            path.display(),
            have,
            current
        ));
        record_next_action(suggested);
    }

    Ok(())
}

pub(crate) fn check_operator_command_dir(
    dir: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
    record_next_action: &mut impl FnMut(&str),
) -> anyhow::Result<()> {
    if !dir.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            dir.display()
        ));
        record_next_action(suggested);
        return Ok(());
    }

    let mut missing = Vec::new();
    let mut missing_version = None;
    for name in [
        "ap-doctor.md",
        "ap-update.md",
        "ap-preview.md",
        "ap-plan.md",
        "ap-deploy.md",
        "ap-status.md",
        "ap-diff.md",
        "ap-explain.md",
        "ap-evolve.md",
    ] {
        let path = dir.join(name);
        if !path.exists() {
            missing.push(name.to_string());
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("read operator asset {}", path.display()))?;
        let Some(have) = extract_agentpack_version(&text) else {
            missing_version = Some(path);
            continue;
        };
        if have != current {
            warnings.push(format!(
                "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
                path.display(),
                have,
                current
            ));
        }
    }

    if let Some(path) = missing_version {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        record_next_action(suggested);
        return Ok(());
    }

    if !missing.is_empty() {
        warnings.push(format!(
            "operator assets incomplete ({location}): missing {}; run: {suggested}",
            missing.join(", "),
        ));
        record_next_action(suggested);
    }

    Ok(())
}
