//! The canonical reference helpers for tenex-edge identities.
//!
//! User-facing session identity is **`sessionCode-agent`** (for example
//! `willow-echo-042-codex`). Backend-qualified agent references (`agent@backend-label`)
//! remain an operator/backend selection syntax for invite and lookup
//! paths, not the public session handle.
//!
//! Rules that hold everywhere:
//!   - Agent session handles are rendered with [`session_handle`].
//!   - Backend labels are config.json `backendName` values and are preserved
//!     exactly after trimming. They are not DNS hostnames and are not slugified.
//!   - Raw session ids are internal correlation handles. A friendly session
//!     codename appears before the agent slug in `sessionCode-agent`.
//!
//! Every renderer formats through this module; every input is classified via
//! [`parse_ref`] or normalized through the session-handle helpers.

/// Canonical user-facing session handle: `sessionCode-agentSlug`.
pub fn session_handle(agent_slug: &str, session: &str) -> String {
    let agent_slug = agent_slug.trim();
    let session = session.trim();
    if session.is_empty() {
        return agent_slug.to_string();
    }
    if agent_slug.is_empty() {
        return session.to_string();
    }
    if let Some((agent, session_ref)) = parse_session_handle(session) {
        if agent == agent_slug {
            return format!("{session_ref}-{agent}");
        }
        return session.to_string();
    }
    if !friendly_session_code(session) {
        return session.to_string();
    }
    format!("{session}-{agent_slug}")
}

/// Parse a public session handle into its two routing-visible parts.
pub fn parse_session_handle(handle: &str) -> Option<(&str, &str)> {
    parse_dashed_session_handle(handle.trim())
}

/// Convert a kind:0 `name` plus tags into the canonical session handle.
pub fn session_handle_from_profile_name(name: &str, agent_slug: &str) -> String {
    let name = name.trim();
    if let Some((agent, session)) = parse_session_handle(name) {
        let agent = if agent_slug.trim().is_empty() {
            agent
        } else {
            agent_slug.trim()
        };
        return session_handle(agent, session);
    }
    if agent_slug.trim().is_empty() {
        name.to_string()
    } else {
        session_handle(agent_slug, name)
    }
}

/// Canonical label for a durable agent: `agent@backend-label`. When the backend
/// label is unknown (empty), degrades to the bare `agent` rather than `agent@`.
pub fn agent_label(slug: &str, host: &str) -> String {
    let host = host.trim();
    if host.is_empty() {
        slug.to_string()
    } else {
        format!("{slug}@{host}")
    }
}

/// Backend-aware agent reference as seen from `local_host`: bare `slug` when the
/// agent is on the local backend (or its backend label is unknown), else
/// `slug@backend-label`.
///
/// This is the token an operator/agent TYPES to address the peer: `developer`
/// names the local developer, while `developer@myBackend` singles out a same-slug
/// agent on another backend. Display layers add the `@` mention sigil on top
/// (`@developer` / `@developer@tower`).
pub fn agent_ref_from(slug: &str, host: &str, local_host: &str) -> String {
    let host = host.trim();
    if host.is_empty() || host == local_host.trim() {
        slug.to_string()
    } else {
        agent_label(slug, host)
    }
}

/// Display for a sender on an envelope "From" line. `slug` is expected to be an
/// already-normalized session handle; callers that provide a bare slug plus
/// host degrade through the backend-qualified form.
pub fn session_label(session_id: &str, slug: &str, host: &str) -> String {
    if slug.is_empty() {
        session_id.to_string()
    } else if let Some((agent, session)) = parse_session_handle(slug) {
        session_handle(agent, session)
    } else {
        agent_label(slug, host)
    }
}

fn parse_dashed_session_handle(handle: &str) -> Option<(&str, &str)> {
    let mut parts = handle.splitn(4, '-');
    let a = parts.next()?;
    let b = parts.next()?;
    let n = parts.next()?;
    let agent_slug = parts.next()?.trim();
    if agent_slug.is_empty() || !friendly_code_parts(a, b, n) {
        return None;
    }
    let session_end = a.len() + b.len() + n.len() + 2;
    Some((agent_slug, &handle[..session_end]))
}

pub(crate) fn looks_like_agent_first_session_handle(handle: &str) -> bool {
    let mut parts = handle.trim().rsplitn(4, '-');
    let Some(n) = parts.next() else { return false };
    let Some(b) = parts.next() else { return false };
    let Some(a) = parts.next() else { return false };
    let Some(agent_slug) = parts.next() else {
        return false;
    };
    !agent_slug.trim().is_empty() && friendly_code_parts(a, b, n)
}

fn friendly_session_code(code: &str) -> bool {
    let mut parts = code.split('-');
    let Some(a) = parts.next() else { return false };
    let Some(b) = parts.next() else { return false };
    let Some(n) = parts.next() else { return false };
    parts.next().is_none() && friendly_code_parts(a, b, n)
}

fn friendly_code_parts(a: &str, b: &str, n: &str) -> bool {
    !a.is_empty()
        && !b.is_empty()
        && a.chars().all(|c| c.is_ascii_alphanumeric())
        && b.chars().all(|c| c.is_ascii_alphanumeric())
        && n.len() == 3
        && n.chars().all(|c| c.is_ascii_digit())
}

/// A short prefix of a message/event id for envelope IDs and send confirmations.
pub fn event_short_id(id: &str) -> String {
    id.chars().take(8).collect()
}

/// A syntactically-classified identifier token (the input side). Resolution
/// against the store happens in the daemon; this is the pure classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ref {
    /// `agent@backend-label` — a durable agent on a specific backend.
    Agent { slug: String, host: String },
    /// A 64-char hex pubkey or `npub1…`.
    Pubkey(String),
    /// Anything else: a session (canonical id / harness alias / id prefix) OR a
    /// bare agent-instance label. The daemon resolver disambiguates by trying
    /// session lookups first, then `slug@<local-host>`.
    Token(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentBackendRef {
    pub slug: String,
    /// Backend label from config.json `backendName`, never a DNS/OS hostname.
    pub backend: Option<String>,
}

/// Parse `agent[@backend-label]` for invite/orchestration surfaces. Unlike
/// display-oriented `agent@backend-label` parsing, the right side is a backend config
/// label and is preserved exactly after trimming.
pub fn parse_agent_backend_ref(spec: &str) -> Option<AgentBackendRef> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }
    match spec.rsplit_once('@') {
        Some((slug, backend)) if !slug.trim().is_empty() && !backend.trim().is_empty() => {
            Some(AgentBackendRef {
                slug: slug.trim().to_string(),
                backend: Some(backend.trim().to_string()),
            })
        }
        Some(_) => None,
        None => Some(AgentBackendRef {
            slug: spec.to_string(),
            backend: None,
        }),
    }
}

/// Parse an identifier token into a syntactic [`Ref`]. `@` ALWAYS means backend
/// label.
pub fn parse_ref(token: &str) -> Ref {
    let t = token.trim();
    if let Some((slug, host)) = t.rsplit_once('@') {
        let slug = slug.trim();
        let host = host.trim();
        if !slug.is_empty() && !host.is_empty() {
            return Ref::Agent {
                slug: slug.to_string(),
                host: host.to_string(),
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

/// Extract inline `@<agent-session-handle>` mentions from free chat text, in
/// order of appearance, deduped. A mention is `@` followed by a run of
/// `[A-Za-z0-9._-]`, with a single internal `@` still accepted for
/// `agent@backend-label`. Trailing punctuation is ignored. Tokens that don't
/// resolve are silently treated as no mention.
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
        // Take the run of handle characters immediately after the '@' sigil,
        // allowing a single internal '@' for host-qualified labels.
        let after = &raw[at + 1..];
        let end = after
            .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '@')))
            .unwrap_or(after.len());
        // Drop trailing separators so `@codex.` and `@haiku@` degrade cleanly.
        let candidate = after[..end].trim_end_matches(['.', '@']);
        if !candidate.is_empty() && !out.iter().any(|m| m == candidate) {
            out.push(candidate.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests;
