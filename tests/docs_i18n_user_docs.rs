use std::path::{Path, PathBuf};

fn collect_user_docs_en() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let dirs = [
        "docs/tutorials",
        "docs/howto",
        "docs/explanation",
        "docs/reference",
    ];
    let excluded = [
        Path::new("docs/reference/json-api.md"),
        Path::new("docs/reference/error-codes.md"),
    ];

    for dir in dirs {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            if path.strip_prefix("docs/zh-CN").is_ok() {
                continue;
            }
            if path.strip_prefix("docs/archive").is_ok() {
                continue;
            }
            if excluded.contains(&path) {
                continue;
            }
            out.push(path.to_path_buf());
        }
    }

    out.sort();
    out
}

fn read_to_string(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn normalize_rel(rel: &Path) -> String {
    rel.to_string_lossy().replace('\\', "/")
}

fn expect_language_header_en(
    en_path: &Path,
    zh_rel: &Path,
    content: &str,
    errors: &mut Vec<String>,
) {
    let rel = normalize_rel(zh_rel);
    let expected = format!("> Language: English | [Chinese (Simplified)](../zh-CN/{rel})");
    if !content.contains(&expected) {
        errors.push(format!(
            "{}: missing or incorrect language header. Expected: {}",
            en_path.display(),
            expected
        ));
    }
}

fn expect_language_header_zh(
    zh_path: &Path,
    en_rel: &Path,
    content: &str,
    errors: &mut Vec<String>,
) {
    let rel = normalize_rel(en_rel);
    let expected = format!("> Language: 简体中文 | [English](../../{rel})");
    if !content.contains(&expected) {
        errors.push(format!(
            "{}: missing or incorrect language header. Expected: {}",
            zh_path.display(),
            expected
        ));
    }
}

#[test]
fn user_docs_have_zh_counters_and_language_links() {
    let mut errors = Vec::new();

    for en_path in collect_user_docs_en() {
        let rel = match en_path.strip_prefix("docs") {
            Ok(r) => r,
            Err(_) => {
                errors.push(format!("{}: not under docs/", en_path.display()));
                continue;
            }
        };
        let zh_path = Path::new("docs/zh-CN").join(rel);
        if !zh_path.exists() {
            errors.push(format!(
                "{}: missing Chinese counterpart at {}",
                en_path.display(),
                zh_path.display()
            ));
            continue;
        }

        if let Some(en_content) = read_to_string(&en_path) {
            expect_language_header_en(&en_path, rel, &en_content, &mut errors);
        } else {
            errors.push(format!("{}: failed to read", en_path.display()));
        }

        if let Some(zh_content) = read_to_string(&zh_path) {
            expect_language_header_zh(&zh_path, rel, &zh_content, &mut errors);
        } else {
            errors.push(format!("{}: failed to read", zh_path.display()));
        }
    }

    let json_api_zh = Path::new("docs/zh-CN/reference/json-api.md");
    if !json_api_zh.exists() {
        errors.push(format!("missing placeholder: {}", json_api_zh.display()));
    }
    let error_codes_zh = Path::new("docs/zh-CN/reference/error-codes.md");
    if !error_codes_zh.exists() {
        errors.push(format!("missing placeholder: {}", error_codes_zh.display()));
    }

    if !errors.is_empty() {
        panic!(
            "Docs i18n checks failed:\n{}",
            errors
                .into_iter()
                .map(|e| format!("- {e}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
