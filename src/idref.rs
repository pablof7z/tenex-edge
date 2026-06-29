//! The SINGLE canonical way to refer to agents and sessions across every
//! tenex-edge interface (identifier standardization).
//!
//! Identity is the durable AGENT-INSTANCE = `(slug, machine)` → **`agent@host`**
//! (e.g. `codex@laptop`, `haiku1@laptop`). A single run (SESSION) is only a
//! correlation handle (the raw `session_id`); it is never a separate display name
//! and is never accepted as a chat target.
//!
//! Rules that hold EVERYWHERE:
//!   - `@` always means HOST, never project. An agent is `(slug, machine)`;
//!     project is only where a message goes, never who it is.
//!   - `host` is ALWAYS slugified for display and matching (`slugify_host`).
//!   - identity is the agent-instance label, resolving to the instance's selected
//!     pubkey; correlation is the raw `session_id`.
//!
//! Every renderer formats via [`agent_label`] / [`session_label`]; every input
//! is classified via [`parse_ref`]. Nothing hand-rolls `format!("{slug}@…")`.

use crate::util::slugify_host;

/// Canonical label for a durable agent: `agent@host` (host slugified). When the
/// host is unknown (empty), degrades to the bare `agent` rather than `agent@`.
pub fn agent_label(slug: &str, host: &str) -> String {
    if host.trim().is_empty() {
        slug.to_string()
    } else {
        format!("{slug}@{}", slugify_host(host))
    }
}

/// Host-aware agent reference as seen from `local_host`: bare `slug` when the
/// agent is on the local machine (or its host is unknown), else `slug@host`.
///
/// This is the token an operator/agent TYPES to address the peer: `developer`
/// names the local developer, while `developer@tower` singles out a same-slug
/// agent on another machine. Display layers add the `@` mention sigil on top
/// (`@developer` / `@developer@tower`).
pub fn agent_ref_from(slug: &str, host: &str, local_host: &str) -> String {
    let host = host.trim();
    if host.is_empty() || slugify_host(host) == slugify_host(local_host) {
        slug.to_string()
    } else {
        agent_label(slug, host)
    }
}

/// Display for a sender on an envelope "From" line: the agent-instance label
/// `agent@host` (host slugified). When the agent slug is unknown, degrades to the
/// raw `session_id` as a bare correlation handle.
pub fn session_label(session_id: &str, slug: &str, host: &str) -> String {
    if slug.is_empty() {
        session_id.to_string()
    } else {
        agent_label(slug, host)
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
    /// Anything else: a session (canonical id / harness alias / id prefix) OR a
    /// bare agent-instance label. The daemon resolver disambiguates by trying
    /// session lookups first, then `slug@<local-host>`.
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

fn is_pubkey(s: &str) -> bool {
    (s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())) || s.starts_with("npub1")
}

/// Extract inline `@<agent-instance-label>` mentions from free chat text, in
/// order of appearance, deduped. A mention is `@` followed by an agent-instance
/// label token — a run of `[A-Za-z0-9._-]`, optionally host-qualified as
/// `label@host` (so both `@haiku1` and `@haiku@laptop` are captured; the resolver
/// understands `agent@host`). Used so `chat write "hey @haiku1"` routes to that
/// instance. Trailing punctuation (`,`, `.` at the end of a word, `!`, `?`, `:`)
/// is ignored. Tokens that don't resolve are silently treated as no mention.
pub fn extract_mentions(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in body.split(|c: char| c.is_whitespace()) {
        let Some(at) = raw.find('@') else { continue };
        if at > 0
            && raw[..at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric())
        {
            continue;
        }
        // Take the run of label/host characters immediately after the '@' sigil,
        // allowing a single internal '@' for a host-qualified `label@host`.
        let after = &raw[at + 1..];
        let end = after
            .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '@')))
            .unwrap_or(after.len());
        // Drop trailing dots/sigils so `@haiku1.` and `@haiku@` degrade cleanly.
        let candidate = after[..end].trim_end_matches(['.', '@']);
        if !candidate.is_empty() && !out.iter().any(|m| m == candidate) {
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
        assert_eq!(
            agent_label("codex", "Pablo's Laptop"),
            "codex@pablo-s-laptop"
        );
        assert_eq!(agent_label("claude", "laptop"), "claude@laptop");
    }

    #[test]
    fn agent_ref_from_is_bare_local_and_qualified_remote() {
        // Same host (after slugify) → bare slug; you'd type just `developer`.
        assert_eq!(agent_ref_from("developer", "laptop", "laptop"), "developer");
        assert_eq!(
            agent_ref_from("developer", "Pablo's Laptop", "pablo-s-laptop"),
            "developer"
        );
        // Unknown host → bare (can't qualify what we don't know).
        assert_eq!(agent_ref_from("developer", "", "laptop"), "developer");
        // Different host → host-qualified so a same-slug remote stays distinct.
        assert_eq!(
            agent_ref_from("developer", "tower", "laptop"),
            "developer@tower"
        );
    }

    #[test]
    fn session_label_is_agent_at_host() {
        // The sender's "From" identity is the agent-instance label.
        assert_eq!(session_label("te-abc-0", "codex", "laptop"), "codex@laptop");
        // Unknown slug degrades to the raw session id as a correlation handle.
        assert_eq!(session_label("te-abc-0", "", "laptop"), "te-abc-0");
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
        assert!(matches!(parse_ref("haiku1"), Ref::Token(_)));
        assert!(matches!(parse_ref("codex"), Ref::Token(_)));
    }

    #[test]
    fn extract_inline_mentions() {
        // Agent-instance labels (bare and ordinal) are now accepted mentions.
        assert_eq!(
            extract_mentions("hey @haiku1 and @codex, look"),
            vec!["haiku1".to_string(), "codex".to_string()]
        );
        // Host-qualified `label@host` is captured intact for the resolver.
        assert_eq!(
            extract_mentions("ping @claude@tower please"),
            vec!["claude@tower".to_string()]
        );
        // Trailing punctuation is trimmed.
        assert_eq!(extract_mentions("ping @codex."), vec!["codex".to_string()]);
        // Email-ish substrings are not mentions.
        assert!(extract_mentions("email dev@example.com please").is_empty());
        // dedup
        assert_eq!(
            extract_mentions("@haiku1 @haiku1"),
            vec!["haiku1".to_string()]
        );
    }
}
