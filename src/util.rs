//! Tiny shared helpers.

use std::time::{SystemTime, UNIX_EPOCH};

/// Current unix time in seconds.
pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A short, human-targetable prefix of an id (for `who` / `--recipient`).
pub fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
}

/// Hash a session ID to a unique, stable 6-character code.
/// Deterministic hash ensures the same session_id always gets the same code.
pub fn session_short_code(session_id: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    session_id.hash(&mut hasher);
    let hash = hasher.finish();

    // Format as 6-char hex for visual distinction and stable output
    format!("{:06x}", hash % 0x1_000_000)
}

/// Convert a human-readable host label (e.g. "pablos' laptop") into a
/// URL-safe slug (e.g. "pablos-laptop") suitable for use in `agent@host`
/// addressing.
pub fn slugify_host(host: &str) -> String {
    let slug: String = host
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse consecutive hyphens and strip leading/trailing ones.
    let mut out = String::with_capacity(slug.len());
    let mut prev_hyphen = true; // treat start as hyphen to strip leading ones
    for c in slug.chars() {
        if c == '-' {
            if !prev_hyphen {
                out.push('-');
                prev_hyphen = true;
            }
        } else {
            out.push(c);
            prev_hyphen = false;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_id_truncates() {
        assert_eq!(short_id("0123456789abcdef"), "01234567");
        assert_eq!(short_id("abc"), "abc");
    }

    #[test]
    fn slugify_host_normalizes() {
        assert_eq!(slugify_host("pablos' laptop"), "pablos-laptop");
        assert_eq!(slugify_host("My MacBook Pro!"), "my-macbook-pro");
        assert_eq!(slugify_host("tower"), "tower");
        assert_eq!(slugify_host("  "), "unknown");
        assert_eq!(slugify_host("abc--def"), "abc-def");
    }

    #[test]
    fn now_is_after_2020() {
        assert!(now_secs() > 1_577_836_800);
    }
}
