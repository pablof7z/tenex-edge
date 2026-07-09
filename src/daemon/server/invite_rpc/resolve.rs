//! Resolve a `channel add --session` selector to a concrete session. Selectors
//! arrive as a raw opaque session id, or — the recruiting-facing form — a
//! `@codename@host` handle. Because `friendly_short_code` is ONE-WAY, a codename
//! is resolved by SCANNING live/known sessions and matching the code, never by
//! inverting it.

use crate::daemon::server::DaemonState;
use crate::state::Session;
use anyhow::Result;
use std::sync::Arc;

pub(super) struct RemoteSession {
    pub(super) session_id: String,
    pub(super) pubkey: String,
    pub(super) slug: String,
    pub(super) backend: String,
}

/// Split a selector into `(codename_or_id, host?)`. A leading `@` sigil is
/// optional; `codename@host` splits on the last `@`. A bare token (raw id or
/// bare codename) carries no host.
fn split_codename_host(selector: &str) -> (String, Option<String>) {
    let s = selector.trim();
    let s = s.strip_prefix('@').unwrap_or(s);
    match s.rsplit_once('@') {
        Some((code, host)) => (code.to_string(), Some(host.to_string())),
        None => (s.to_string(), None),
    }
}

/// A LOCAL session for the selector: first the existing id/prefix match, then a
/// codename scan over live sessions. A selector that names a non-local host is
/// never matched here.
pub(super) fn local_session(state: &Arc<DaemonState>, selector: &str) -> Option<Session> {
    if let Some(rec) = state
        .with_store(|s| s.get_session(selector))
        .ok()
        .flatten()
        .or_else(|| {
            state
                .with_store(|s| s.find_session_by_prefix(selector))
                .ok()
                .flatten()
        })
    {
        return Some(rec);
    }
    let (codename, host) = split_codename_host(selector);
    // A host that names another backend means the caller wants a remote session.
    if host
        .as_deref()
        .is_some_and(|h| !h.is_empty() && h != state.host)
    {
        return None;
    }
    state.with_store(|s| {
        s.list_alive_sessions()
            .unwrap_or_default()
            .into_iter()
            .find(|rec| crate::util::friendly_short_code(&rec.session_id) == codename)
    })
}

/// A REMOTE session for the selector, from the materialized status cache. Matches
/// a raw id (exact/prefix) or a `@codename@host` handle (code scan honoring the
/// host so two backends' codenames cannot collide).
pub(super) fn remote_session_from_status(
    state: &Arc<DaemonState>,
    selector: &str,
) -> Result<RemoteSession> {
    let (codename, want_host) = split_codename_host(selector);
    let matches = state.with_store(|s| -> Result<Vec<RemoteSession>> {
        let mut out = Vec::new();
        for st in s.list_status_sessions(None, None)? {
            let by_id = st.session_id == selector || st.session_id.starts_with(selector);
            let by_code = crate::util::friendly_short_code(&st.session_id) == codename;
            if !by_id && !by_code {
                continue;
            }
            let Some(profile) = s.get_profile(&st.pubkey)? else {
                continue;
            };
            // When resolving by codename with an explicit host, only that backend's
            // session qualifies.
            if by_code && !by_id {
                if let Some(h) = want_host.as_deref().filter(|h| !h.is_empty()) {
                    if profile.host != h {
                        continue;
                    }
                }
            }
            let slug = if profile.slug.is_empty() {
                st.slug.clone()
            } else {
                profile.slug.clone()
            };
            out.push(RemoteSession {
                session_id: st.session_id,
                pubkey: st.pubkey,
                slug,
                backend: profile.host,
            });
        }
        out.sort_by(|a, b| {
            a.session_id
                .cmp(&b.session_id)
                .then(a.pubkey.cmp(&b.pubkey))
        });
        out.dedup_by(|a, b| a.session_id == b.session_id && a.pubkey == b.pubkey);
        Ok(out)
    })?;
    match matches.as_slice() {
        [one] => Ok(RemoteSession {
            session_id: one.session_id.clone(),
            pubkey: one.pubkey.clone(),
            slug: one.slug.clone(),
            backend: one.backend.clone(),
        }),
        [] => anyhow::bail!("no session matching {selector:?}"),
        _ => anyhow::bail!(
            "session {selector:?} is ambiguous; use the full session id or @codename@host"
        ),
    }
}
