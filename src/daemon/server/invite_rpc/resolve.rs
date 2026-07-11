//! Resolve `channel add --session` by permanent npub/hex identity or by the
//! exact current leased handle. Raw session ids are internal and never accepted.

use crate::daemon::server::DaemonState;
use crate::state::Session;
use anyhow::Result;
use std::sync::Arc;

pub(super) struct RemoteSession {
    pub(super) pubkey: String,
    pub(super) slug: String,
    pub(super) backend: String,
}

pub(super) fn local_session(state: &Arc<DaemonState>, selector: &str) -> Option<Session> {
    let selector = selector.trim().trim_start_matches('@');
    let pubkey = crate::idref::normalize_pubkey(selector).or_else(|| {
        state
            .with_store(|s| s.pubkey_for_handle(selector))
            .ok()
            .flatten()
    })?;
    state
        .with_store(|s| s.session_for_pubkey(&pubkey))
        .ok()
        .flatten()
}

pub(super) fn remote_session_from_status(
    state: &Arc<DaemonState>,
    selector: &str,
) -> Result<RemoteSession> {
    let selector = selector.trim().trim_start_matches('@');
    let selected_pubkey = crate::idref::normalize_pubkey(selector);
    let now = crate::util::now_secs();
    let matches = state.with_store(|s| -> Result<Vec<RemoteSession>> {
        let mut out = Vec::new();
        for st in s.list_status_sessions(None, None)? {
            let Some(profile) = s.get_profile(&st.pubkey)? else {
                continue;
            };
            let matches = selected_pubkey
                .as_deref()
                .is_some_and(|pubkey| pubkey == st.pubkey)
                || (selected_pubkey.is_none()
                    && st.expiration >= now
                    && (profile.name == selector || profile.slug == selector));
            if !matches {
                continue;
            }
            out.push(RemoteSession {
                pubkey: st.pubkey,
                slug: profile.slug,
                backend: profile.host,
            });
        }
        out.sort_by(|a, b| a.pubkey.cmp(&b.pubkey));
        out.dedup_by(|a, b| a.pubkey == b.pubkey);
        Ok(out)
    })?;
    match matches.as_slice() {
        [one] => Ok(RemoteSession {
            pubkey: one.pubkey.clone(),
            slug: one.slug.clone(),
            backend: one.backend.clone(),
        }),
        [] => anyhow::bail!("no session matching {selector:?}; use its npub or current handle"),
        _ => anyhow::bail!("session selector is ambiguous; use the full npub"),
    }
}
