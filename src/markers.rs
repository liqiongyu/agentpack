use std::collections::BTreeMap;

use anyhow::Context as _;

pub const MODULE_SECTION_START_PREFIX: &str = "<!-- agentpack:module=";
pub const MODULE_SECTION_END_MARKER: &str = "<!-- /agentpack -->";

pub fn format_module_section(module_id: &str, content: &str) -> String {
    let mut out = String::new();
    out.push_str(MODULE_SECTION_START_PREFIX);
    out.push_str(module_id);
    out.push_str(" -->\n");
    out.push_str(content);
    if !content.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(MODULE_SECTION_END_MARKER);
    out
}

pub fn parse_module_sections(text: &str) -> anyhow::Result<BTreeMap<String, String>> {
    fn parse_start_marker(line: &str) -> Option<String> {
        let trimmed = line.trim();
        if !trimmed.starts_with(MODULE_SECTION_START_PREFIX) {
            return None;
        }
        if !trimmed.ends_with("-->") {
            return None;
        }

        let raw = trimmed
            .strip_prefix(MODULE_SECTION_START_PREFIX)?
            .strip_suffix("-->")?
            .trim();
        if raw.is_empty() {
            return None;
        }
        Some(raw.to_string())
    }

    let mut sections = BTreeMap::<String, String>::new();
    let mut current: Option<(String, String)> = None;

    for line in text.split_inclusive('\n') {
        if let Some((_module_id, buf)) = current.as_mut() {
            if line.trim() == MODULE_SECTION_END_MARKER {
                let Some((module_id, buf)) = current.take() else {
                    continue;
                };
                anyhow::ensure!(
                    !sections.contains_key(&module_id),
                    "duplicate module section: {module_id}"
                );
                sections.insert(module_id, buf);
                continue;
            }

            if parse_start_marker(line).is_some() {
                anyhow::bail!("nested module section marker");
            }

            buf.push_str(line);
            continue;
        }

        if let Some(module_id) = parse_start_marker(line) {
            current = Some((module_id, String::new()));
        }
    }

    if let Some((module_id, _)) = current {
        anyhow::bail!("unterminated module section for {module_id}");
    }

    Ok(sections)
}

pub fn parse_module_sections_from_bytes(bytes: &[u8]) -> anyhow::Result<BTreeMap<String, String>> {
    let text = std::str::from_utf8(bytes).context("decode as utf-8")?;
    parse_module_sections(text)
}
