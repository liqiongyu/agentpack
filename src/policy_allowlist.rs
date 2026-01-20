pub(crate) fn normalize_git_remote_for_policy(url: &str) -> String {
    let mut u = url.trim().trim_end_matches(".git").to_string();
    if let Some(rest) = u.strip_prefix("git@") {
        u = rest.replace(':', "/");
    } else if let Some(rest) = u.strip_prefix("https://") {
        u = rest.to_string();
    } else if let Some(rest) = u.strip_prefix("http://") {
        u = rest.to_string();
    } else if let Some(rest) = u.strip_prefix("ssh://") {
        u = rest.to_string();
        if let Some((_, rest)) = u.split_once('@') {
            u = rest.to_string();
        }
        u = u.replace(':', "/");
    }
    u.trim_start_matches('/').to_lowercase()
}

pub(crate) fn remote_matches_allowlist(normalized_remote: &str, normalized_allow: &str) -> bool {
    if normalized_allow.is_empty() {
        return false;
    }
    if normalized_remote == normalized_allow {
        return true;
    }
    if !normalized_remote.starts_with(normalized_allow) {
        return false;
    }
    if normalized_allow.ends_with('/') {
        return true;
    }
    normalized_remote
        .as_bytes()
        .get(normalized_allow.len())
        .copied()
        == Some(b'/')
}
