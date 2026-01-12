use crate::hash::sha256_hex;

pub fn is_safe_legacy_path_component(value: &str) -> bool {
    if value.is_empty() || value == "." || value == ".." {
        return false;
    }
    if value.contains('/') || value.contains('\\') {
        return false;
    }

    if cfg!(windows)
        && (value.contains(':')
            || value.contains('*')
            || value.contains('?')
            || value.contains('"')
            || value.contains('<')
            || value.contains('>')
            || value.contains('|'))
    {
        return false;
    }

    true
}

pub fn sanitize_fs_component(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            _ if c.is_ascii_alphanumeric() || c == '-' || c == '_' => c,
            _ => '_',
        })
        .collect()
}

pub fn module_fs_key(module_id: &str) -> String {
    let sanitized = sanitize_fs_component(module_id);
    let hash = sha256_hex(module_id.as_bytes());
    format!("{sanitized}--{}", &hash[..10])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_fs_key_is_stable_and_collision_resistant() {
        let a = module_fs_key("a:b");
        let b = module_fs_key("a/b");
        assert_ne!(a, b);
        assert!(a.contains("--"));
        assert!(b.contains("--"));
    }

    #[test]
    fn module_fs_key_sanitizes_for_filesystems() {
        let key = module_fs_key("instructions:base");
        assert!(key.starts_with("instructions_base--"));
        assert!(!key.contains(':'));
        assert!(!key.contains('/'));
        assert!(!key.contains('\\'));
    }

    #[test]
    fn legacy_component_safety_matches_platform_constraints() {
        if cfg!(windows) {
            assert!(!is_safe_legacy_path_component("instructions:base"));
        } else {
            assert!(is_safe_legacy_path_component("instructions:base"));
        }
        assert!(!is_safe_legacy_path_component(""));
        assert!(!is_safe_legacy_path_component(".."));
        assert!(!is_safe_legacy_path_component("a/b"));
    }
}
