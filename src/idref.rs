//! The SINGLE canonical way to refer to agents and sessions across every
//! tenex-edge interface (identifier standardization).
//!
//! Two identifier kinds, and exactly two display forms:
//!   • durable AGENT = `(slug, machine)` → **`agent@host`**   e.g. `codex@laptop`
//!   • SESSION (one run) = a `codename`   → displayed as
//!     **`codename (agent@host)`**         e.g. `bravo4217 (codex@laptop)`
//!
//! Rules that hold EVERYWHERE:
//!   - `@` always means HOST, never project. An agent is `(slug, machine)`;
//!     project is only where a message goes, never who it is.
//!   - `host` is ALWAYS slugified for display and matching (`slugify_host`).
//!   - a session is shown as `codename (agent@host)`, codename first.
//!
//! Every renderer formats via [`agent_label`] / [`session_label`]; every input
//! is classified via [`parse_ref`]. Nothing hand-rolls `format!("{slug}@…")`.

use crate::util::{looks_like_codename, session_codename, slugify_host};

/// Canonical label for a durable agent: `agent@host` (host slugified). When the
/// host is unknown (empty), degrades to the bare `agent` rather than `agent@`.
pub fn agent_label(slug: &str, host: &str) -> String {
    if host.trim().is_empty() {
        slug.to_string()
    } else {
        format!("{slug}@{}", slugify_host(host))
    }
}

/// Canonical display for a session: `codename (agent@host)`. Degrades to
/// `codename (agent)` when host is unknown, or bare `codename` when slug is too.
pub fn session_label(session_id: &str, slug: &str, host: &str) -> String {
    let code = session_codename(session_id);
    if slug.is_empty() {
        code
    } else {
        format!("{code} ({})", agent_label(slug, host))
    }
}

/// A syntactically-classified identifier token (the input side). Resolution
/// against the store happens in the daemon; this is the pure classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ref {
    /// `agent@host` — a durable agent on a specific machine (host slugified).
    Agent { slug: String, host: String },
    /// A 64-char hex pubkey or `npub1…`.
    Pubkey(String),
    /// Anything else: a session (canonical id / harness alias / id prefix /
    /// codename) OR a bare agent slug. The daemon resolver disambiguates by
    /// trying session lookups first, then `slug@<local-host>`.
    Token(String),
}

/// Parse an identifier token into a syntactic [`Ref`]. `@` ALWAYS means host.
pub fn parse_ref(token: &str) -> Ref {
    let t = token.trim();
    if let Some((slug, host)) = t.rsplit_once('@') {
        if !slug.is_empty() && !host.is_empty() {
            return Ref::Agent {
                slug: slug.to_string(),
                host: slugify_host(host),
            };
        }
    }
    if is_pubkey(t) {
        return Ref::Pubkey(t.to_string());
    }
    Ref::Token(t.to_string())
}

/// True if `token` is best understood as a session reference (a codename) vs a
/// bare agent slug — used by the resolver to order its lookups.
pub fn token_is_codename(token: &str) -> bool {
    looks_like_codename(token.trim())
}

fn is_pubkey(s: &str) -> bool {
    (s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())) || s.starts_with("npub1")
}

/// Extract inline `@codename` mentions from free chat text, in order of
/// appearance, deduped. A mention is `@` followed by a codename-shaped token
/// (`<nato-word><digits>`). Used so `chat write "hey @bravo4217"` highlights the
/// session. Punctuation after the codename (`,`, `.`, `!`, `?`, `:`) is ignored.
pub fn extract_mentions(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in body.split(|c: char| c.is_whitespace()) {
        let Some(at) = raw.find('@') else { continue };
        // Take the run of [A-Za-z0-9] immediately after '@'.
        let after = &raw[at + 1..];
        let end = after
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(after.len());
        let candidate = &after[..end];
        if looks_like_codename(candidate) && !out.iter().any(|m| m == candidate) {
            out.push(candidate.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_label_slugifies_host() {
        assert_eq!(agent_label("codex", "Pablo's Laptop"), "codex@pablo-s-laptop");
        assert_eq!(agent_label("claude", "laptop"), "claude@laptop");
    }

    #[test]
    fn session_label_is_codename_then_agent_at_host() {
        let s = session_label("te-abc-0", "codex", "laptop");
        // codename first, then (agent@host)
        assert!(s.ends_with(" (codex@laptop)"), "{s}");
        assert!(!s.starts_with("codex"), "{s}");
    }

    #[test]
    fn parse_at_is_host_not_project() {
        match parse_ref("codex@laptop") {
            Ref::Agent { slug, host } => {
                assert_eq!(slug, "codex");
                assert_eq!(host, "laptop");
            }
            other => panic!("{other:?}"),
        }
        // host gets slugified
        match parse_ref("codex@Pablo's Laptop") {
            Ref::Agent { host, .. } => assert_eq!(host, "pablo-s-laptop"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parse_pubkey_and_token() {
        let hex = "a".repeat(64);
        assert!(matches!(parse_ref(&hex), Ref::Pubkey(_)));
        assert!(matches!(parse_ref("npub1abcdef"), Ref::Pubkey(_)));
        assert!(matches!(parse_ref("bravo4217"), Ref::Token(_)));
        assert!(matches!(parse_ref("codex"), Ref::Token(_)));
    }

    #[test]
    fn codename_classification() {
        assert!(token_is_codename("bravo4217"));
        assert!(token_is_codename("Echo0163"));
        assert!(!token_is_codename("codex"));
        assert!(!token_is_codename("router5")); // not a NATO word
    }

    #[test]
    fn extract_inline_mentions() {
        assert_eq!(
            extract_mentions("hey @bravo4217 and @echo0163, look"),
            vec!["bravo4217".to_string(), "echo0163".to_string()]
        );
        // bare email-ish / non-codename @ is ignored
        assert!(extract_mentions("ping @codex please").is_empty());
        // dedup
        assert_eq!(
            extract_mentions("@bravo4217 @bravo4217"),
            vec!["bravo4217".to_string()]
        );
    }
}
