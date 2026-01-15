use crate::user_error::UserError;

pub(crate) fn print_user_error_human(err: &anyhow::Error) -> bool {
    let Some(user_err) = err.chain().find_map(|e| e.downcast_ref::<UserError>()) else {
        return false;
    };

    match user_err.code.as_str() {
        "E_DESIRED_STATE_CONFLICT" => {
            eprintln!("error[{}]: {}", user_err.code, user_err.message);

            let Some(details) = user_err.details.as_ref() else {
                return true;
            };

            let target = details
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            let path = details
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            let existing_sha = details.pointer("/existing/sha256").and_then(|v| v.as_str());
            let new_sha = details.pointer("/new/sha256").and_then(|v| v.as_str());
            let existing_modules =
                summarize_json_string_array(details.pointer("/existing/module_ids"), 4);
            let new_modules = summarize_json_string_array(details.pointer("/new/module_ids"), 4);

            eprintln!("  target: {target}");
            eprintln!("  path: {path}");
            if let Some(sha) = existing_sha {
                eprintln!("  existing sha256: {sha}");
            }
            eprintln!("  existing modules: {existing_modules}");
            if let Some(sha) = new_sha {
                eprintln!("  new sha256: {sha}");
            }
            eprintln!("  new modules: {new_modules}");
            eprintln!(
                "hint: ensure only one module owns each target/path, or make the contents identical."
            );

            true
        }
        "E_POLICY_VIOLATIONS" => {
            eprintln!("error[{}]: {}", user_err.code, user_err.message);

            let Some(details) = user_err.details.as_ref() else {
                return true;
            };

            let root = details
                .get("root")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            let violations = details
                .pointer("/summary/violations")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            eprintln!("  root: {root}");
            eprintln!("  violations: {violations}");

            let Some(issues) = details.get("issues").and_then(|v| v.as_array()) else {
                return true;
            };
            for issue in issues {
                let rule = issue
                    .get("rule")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>");
                let path = issue
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>");
                let message = issue
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>");
                let line = issue.pointer("/details/line").and_then(|v| v.as_u64());
                if let Some(line) = line {
                    eprintln!("  - [{rule}] {path}:{line}: {message}");
                } else {
                    eprintln!("  - [{rule}] {path}: {message}");
                }
            }

            true
        }
        _ => false,
    }
}

fn summarize_json_string_array(value: Option<&serde_json::Value>, max: usize) -> String {
    let mut unique = std::collections::BTreeSet::new();
    if let Some(values) = value.and_then(|v| v.as_array()) {
        for v in values {
            if let Some(s) = v.as_str() {
                unique.insert(s.to_string());
            }
        }
    }

    if unique.is_empty() {
        return "<none>".to_string();
    }

    let total = unique.len();
    let shown: Vec<String> = unique.into_iter().take(max).collect();
    let remaining = total.saturating_sub(shown.len());

    let mut out = shown.join(", ");
    if remaining > 0 {
        out.push_str(&format!(" ...({} more)", remaining));
    }

    out
}
