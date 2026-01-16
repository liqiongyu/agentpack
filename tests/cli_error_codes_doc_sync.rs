use std::collections::BTreeSet;

fn codes_from_error_codes_md() -> anyhow::Result<BTreeSet<String>> {
    let text = std::fs::read_to_string("docs/ERROR_CODES.md")?;
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("### ") else {
            continue;
        };
        let code = rest.split_whitespace().next().unwrap_or_default();
        if code.starts_with("E_") {
            out.insert(code.to_string());
        }
    }
    Ok(out)
}

fn codes_from_user_error_new_calls() -> anyhow::Result<BTreeSet<String>> {
    let mut out = BTreeSet::new();

    for entry in walkdir::WalkDir::new("src") {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let text = std::fs::read_to_string(entry.path())?;
        extract_codes_for_callsite(&text, "UserError::new(", &mut out);

        if entry.path().ends_with("user_error.rs") {
            extract_codes_for_callsite(&text, "Self::new(", &mut out);
        }
    }

    // `--json` fallback code for non-UserError failures.
    out.insert("E_UNEXPECTED".to_string());

    Ok(out)
}

fn extract_codes_for_callsite(text: &str, needle: &str, out: &mut BTreeSet<String>) {
    let mut idx = 0;
    while let Some(pos) = text[idx..].find(needle) {
        idx += pos + needle.len();

        let rest = &text[idx..];
        let mut offset = 0;
        for ch in rest.chars() {
            if ch.is_whitespace() {
                offset += ch.len_utf8();
                continue;
            }
            if ch != '"' {
                break;
            }
            offset += ch.len_utf8();
            let after_quote = &rest[offset..];
            let Some(end) = after_quote.find('"') else {
                break;
            };
            let code = &after_quote[..end];
            if code.starts_with("E_") {
                out.insert(code.to_string());
            }
            break;
        }
        idx += offset;
    }
}

#[test]
fn error_codes_md_matches_emitted_json_codes() -> anyhow::Result<()> {
    let docs = codes_from_error_codes_md()?;
    let code = codes_from_user_error_new_calls()?;

    let missing: Vec<String> = code.difference(&docs).cloned().collect();
    let extra: Vec<String> = docs.difference(&code).cloned().collect();

    if missing.is_empty() && extra.is_empty() {
        return Ok(());
    }

    anyhow::bail!(
        "docs/ERROR_CODES.md is out of sync with emitted error codes.\n\nMissing in docs/ERROR_CODES.md:\n  {}\n\nExtra in docs/ERROR_CODES.md:\n  {}\n\nRemediation:\n- Add missing codes under the appropriate section (Stable vs Non-stable).\n- Remove extra codes if they are no longer emitted.\n",
        if missing.is_empty() {
            "(none)".to_string()
        } else {
            missing.join("\n  ")
        },
        if extra.is_empty() {
            "(none)".to_string()
        } else {
            extra.join("\n  ")
        }
    );
}
