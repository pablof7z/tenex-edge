use crate::daemon::server::DaemonState;
use anyhow::Result;
use std::sync::Arc;

pub(super) fn reserve(state: &Arc<DaemonState>, slug: &str) -> Result<Option<String>> {
    let identity = crate::identity::load_or_create(
        &crate::config::edge_home(),
        slug,
        crate::util::now_secs(),
    )?;
    let conflict = state.with_store(|store| {
        let session = store.list_alive_sessions()?.into_iter().find(|session| {
            session.agent_slug == slug
                && (!identity.per_session_key
                    || store
                        .is_durable_agent_session(&session.session_id)
                        .unwrap_or(false))
        });
        let Some(session) = session else {
            return Ok(None);
        };
        let channels = store
            .list_session_joined_channels(&session.session_id)?
            .into_iter()
            .map(|(channel, _)| channel)
            .collect::<Vec<_>>();
        let pty = store
            .aliases_for_session(&session.session_id)?
            .into_iter()
            .find(|alias| alias.external_id_kind == "pty_session")
            .map(|alias| alias.external_id);
        Ok::<_, anyhow::Error>(Some((session, channels, pty)))
    })?;
    if let Some((session, channels, pty)) = conflict {
        let channels = if channels.is_empty() {
            "<none>".to_string()
        } else {
            channels.join(", ")
        };
        let attach = pty
            .map(|_| "tenex-edge sessions".to_string())
            .unwrap_or_else(|| "tenex-edge sessions".to_string());
        let pubkey = identity.pubkey_hex();
        let npub = crate::idref::npub(&pubkey).unwrap_or(pubkey);
        anyhow::bail!(
            "durable agent {slug:?} already has live session {} in channel(s) {}; \
             attach with `{attach}`, or add it to another channel with \
             `tenex-edge channel add --session {npub} <channel>`",
            session.session_id,
            channels
        );
    }
    if identity.per_session_key {
        return Ok(None);
    }
    let reservation = crate::state::mint_session_id();
    state.with_store(|store| {
        store.claim_durable_agent_session(
            &identity.pubkey_hex(),
            slug,
            &reservation,
            crate::util::now_secs(),
        )
    })?;
    Ok(Some(reservation))
}

pub(super) fn release(state: &Arc<DaemonState>, reservation: Option<&str>) {
    if let Some(reservation) = reservation {
        state
            .with_store(|store| store.release_durable_agent_session(reservation))
            .ok();
    }
}
