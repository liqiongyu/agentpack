use std::collections::BTreeMap;

use anyhow::Context as _;

use crate::config::{GitSource, LocalPathSource, Source};

pub fn parse_source_spec(spec: &str) -> anyhow::Result<Source> {
    if let Some(rest) = spec.strip_prefix("local:") {
        return Ok(Source {
            local_path: Some(LocalPathSource {
                path: rest.to_string(),
            }),
            git: None,
        });
    }

    if let Some(rest) = spec.strip_prefix("git:") {
        let (url, query) = rest.split_once('#').unwrap_or((rest, ""));
        let params = parse_query(query)?;
        let git = GitSource {
            url: url.to_string(),
            ref_name: params
                .get("ref")
                .cloned()
                .unwrap_or_else(|| "main".to_string()),
            subdir: params.get("subdir").cloned().unwrap_or_default(),
            shallow: params
                .get("shallow")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        };
        return Ok(Source {
            local_path: None,
            git: Some(git),
        });
    }

    anyhow::bail!("unsupported source spec (expected local:... or git:...): {spec}");
}

fn parse_query(query: &str) -> anyhow::Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    if query.trim().is_empty() {
        return Ok(out);
    }

    for part in query.split('&') {
        if part.trim().is_empty() {
            continue;
        }
        let (k, v) = part
            .split_once('=')
            .with_context(|| format!("invalid query segment: {part}"))?;
        out.insert(k.to_string(), v.to_string());
    }

    Ok(out)
}
