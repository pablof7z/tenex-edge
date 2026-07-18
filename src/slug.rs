//! Turning free-text display names into safe, restricted-charset slugs.

/// Lowercase `input`, map every non-alphanumeric byte to `-`, then collapse
/// consecutive hyphens and strip leading/trailing ones.
fn collapse_to_slug_chars(input: &str) -> String {
    let slug: String = input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
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
    out
}

/// Convert a human-readable host label (e.g. "pablos' laptop") into a URL-safe
/// slug (e.g. "pablos-laptop"). This is only for internal normalization;
/// public agent/backend labels preserve config.json `backendName`.
pub fn slugify_host(host: &str) -> String {
    let out = collapse_to_slug_chars(host);
    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

/// Convert an arbitrary display name (e.g. a cross-harness agent profile's
/// free-text `name:`, which may contain spaces or punctuation) into the
/// `[a-z0-9-]` subset accepted by [`crate::identity::is_valid_slug`].
pub fn slugify(value: &str) -> String {
    let out = collapse_to_slug_chars(value);
    if out.is_empty() {
        "agent".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_host_normalizes() {
        assert_eq!(slugify_host("pablos' laptop"), "pablos-laptop");
        assert_eq!(slugify_host("My MacBook Pro!"), "my-macbook-pro");
        assert_eq!(slugify_host("tower"), "tower");
        assert_eq!(slugify_host("  "), "unknown");
        assert_eq!(slugify_host("abc--def"), "abc-def");
    }

    #[test]
    fn slugify_normalizes_free_text_agent_names() {
        assert_eq!(slugify("Ava Chen"), "ava-chen");
        assert_eq!(slugify("Remy (Remington)"), "remy-remington");
        assert_eq!(slugify("writer"), "writer");
        assert_eq!(slugify("  "), "agent");
    }
}
