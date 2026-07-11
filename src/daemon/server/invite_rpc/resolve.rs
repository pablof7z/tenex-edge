//! Resolve a `channel add --session` selector to a concrete session. Selectors
//! arrive as a raw opaque session id, or — the recruiting-facing form — an
//! `@sessionCode-agent` handle.

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

/// A LOCAL session for the selector: public `sessionCode-agent`, then raw id/prefix.
pub(super) fn local_session(state: &Arc<DaemonState>, selector: &str) -> Option<Session> {
    let selector = selector.trim().strip_prefix('@').unwrap_or(selector.trim());
    if let Some(rec) = local_session_by_public_handle(state, selector) {
        return Some(rec);
    }
    state
        .with_store(|s| s.get_session(selector))
        .ok()
        .flatten()
        .or_else(|| {
            state
                .with_store(|s| s.find_session_by_prefix(selector))
                .ok()
                .flatten()
        })
}

fn local_session_by_public_handle(state: &Arc<DaemonState>, selector: &str) -> Option<Session> {
    let selector = selector.trim().strip_prefix('@').unwrap_or(selector.trim());
    let (agent, session_ref) = crate::idref::parse_session_handle(selector)?;
    state.with_store(|s| {
        s.list_alive_sessions()
            .unwrap_or_default()
            .into_iter()
            .find(|rec| {
                if rec.agent_slug != agent {
                    return false;
                }
                rec.session_id == session_ref
                    || rec.session_id.starts_with(session_ref)
                    || crate::util::friendly_short_code(&rec.session_id) == session_ref
            })
    })
}

/// A REMOTE session for the selector, from the materialized status cache. Matches
/// `@sessionCode-agent` or a raw id (exact/prefix).
pub(super) fn remote_session_from_status(
    state: &Arc<DaemonState>,
    selector: &str,
) -> Result<RemoteSession> {
    let selector = selector.trim().strip_prefix('@').unwrap_or(selector.trim());
    if let Some((agent, session_ref)) = crate::idref::parse_session_handle(selector) {
        return remote_session_from_public_handle(state, selector, agent, session_ref);
    }
    let matches = state.with_store(|s| -> Result<Vec<RemoteSession>> {
        let mut out = Vec::new();
        for st in s.list_status_sessions(None, None)? {
            let by_id = st.session_id == selector || st.session_id.starts_with(selector);
            if !by_id {
                continue;
            }
            let Some(profile) = s.get_profile(&st.pubkey)? else {
                continue;
            };
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
            "session {selector:?} is ambiguous; use the full session id or @sessionCode-agent"
        ),
    }
}

fn remote_session_from_public_handle(
    state: &Arc<DaemonState>,
    selector: &str,
    agent: &str,
    session_ref: &str,
) -> Result<RemoteSession> {
    let matches = state.with_store(|s| -> Result<Vec<RemoteSession>> {
        let mut out = Vec::new();
        for st in s.list_status_sessions(None, None)? {
            let profile = s.get_profile(&st.pubkey)?;
            let profile_agent = profile
                .as_ref()
                .map(|p| p.agent_slug.as_str())
                .filter(|slug| !slug.is_empty())
                .or_else(|| crate::idref::parse_session_handle(&st.slug).map(|(slug, _)| slug));
            if profile_agent != Some(agent) {
                continue;
            }
            let by_session = st.session_id == session_ref
                || st.session_id.starts_with(session_ref)
                || crate::util::friendly_short_code(&st.session_id) == session_ref;
            if !by_session {
                continue;
            }
            let backend = profile.as_ref().map(|p| p.host.clone()).unwrap_or_default();
            let slug = crate::idref::session_handle(
                agent,
                &crate::util::friendly_short_code(&st.session_id),
            );
            out.push(RemoteSession {
                session_id: st.session_id,
                pubkey: st.pubkey,
                slug,
                backend,
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
            "session {selector:?} is ambiguous; use the full sessionCode-agent handle"
        ),
    }
}
