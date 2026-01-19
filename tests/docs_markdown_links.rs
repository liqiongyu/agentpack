use std::path::{Component, Path, PathBuf};

fn should_skip_target(target: &str) -> bool {
    let target = target.trim();
    if target.is_empty() {
        return true;
    }

    if target.starts_with('#') {
        return true;
    }

    if let Some(scheme) = target.split(':').next() {
        if matches!(scheme, "http" | "https" | "mailto") {
            return true;
        }
    }

    false
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn resolve_link_target(source_path: &Path, raw_target: &str) -> Option<PathBuf> {
    let mut target = raw_target.trim();
    if target.starts_with('<') && target.ends_with('>') && target.len() >= 2 {
        target = &target[1..target.len() - 1];
    }

    let target = target
        .split('#')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .trim();

    if should_skip_target(target) {
        return None;
    }

    let target = target.split_whitespace().next().unwrap_or("").trim();
    if target.is_empty() || should_skip_target(target) {
        return None;
    }

    let base = if let Some(stripped) = target.strip_prefix('/') {
        PathBuf::from(stripped)
    } else {
        let dir = source_path.parent().unwrap_or_else(|| Path::new("."));
        dir.join(target)
    };

    Some(normalize_path(&base))
}

fn extract_link_targets(markdown: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_code_fence = false;

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }

        let mut rest = line;
        while let Some(idx) = rest.find("](") {
            let after = &rest[idx + 2..];
            let Some(end) = after.find(')') else {
                break;
            };
            let target = after[..end].trim().to_string();
            out.push(target);
            rest = &after[end + 1..];
        }
    }

    out
}

fn target_exists(resolved: &Path) -> bool {
    if resolved.exists() {
        return true;
    }
    if resolved.extension().is_none() && resolved.with_extension("md").exists() {
        return true;
    }
    false
}

#[test]
fn docs_links_do_not_point_to_missing_files() {
    let mut broken = Vec::new();

    for entry in walkdir::WalkDir::new("docs")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        if path.strip_prefix("docs/archive").is_ok() {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(err) => {
                broken.push(format!("{}: failed to read: {err}", path.display()));
                continue;
            }
        };

        for raw_target in extract_link_targets(&content) {
            let Some(resolved) = resolve_link_target(path, &raw_target) else {
                continue;
            };
            if !target_exists(&resolved) {
                broken.push(format!(
                    "{}: broken link target '{}' (resolved: {})",
                    path.display(),
                    raw_target,
                    resolved.display()
                ));
            }
        }
    }

    if !broken.is_empty() {
        broken.sort();
        panic!(
            "Found broken markdown links:\n{}",
            broken
                .into_iter()
                .map(|s| format!("- {s}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
