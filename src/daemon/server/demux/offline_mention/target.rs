use crate::daemon::server::DaemonState;
use crate::util::now_secs;
use std::sync::Arc;

pub(super) struct MentionTarget {
    pub(super) agent_slug: String,
    pub(super) session: Option<crate::state::Session>,
}

pub(super) fn resolve_and_persist(
    state: &Arc<DaemonState>,
    event_id: &str,
    mentioned_pubkey: &str,
    channel: &str,
    body: &str,
    requester_pubkey: Option<&str>,
) -> Option<MentionTarget> {
    let backend_pubkey = state.backend_pubkey();
    let claim = state
        .with_store(|store| {
            store
                .get_session_claim(mentioned_pubkey, channel)
                .ok()
                .flatten()
        })
        .filter(|claim| claim.is_owned_by_backend(backend_pubkey.as_deref()));
    let active_claim = state.with_store(|store| {
        store
            .get_active_session_claim(mentioned_pubkey, channel, now_secs())
            .ok()
            .flatten()
    });
    if let Some(remote_claim) = active_claim
        .as_ref()
        .filter(|claim| !claim.is_owned_by_backend(backend_pubkey.as_deref()))
    {
        tracing::info!(
            agent = %remote_claim.agent_slug,
            channel,
            owner_host = %remote_claim.owner_host,
            owner_backend = %crate::util::pubkey_short(&remote_claim.owner_backend_pubkey),
            "active ephemeral session claim belongs to another backend - skipping local recovery"
        );
        return None;
    }
    let active_claim =
        active_claim.filter(|claim| claim.is_owned_by_backend(backend_pubkey.as_deref()));
    let session = match state.with_store(|store| store.get_session(mentioned_pubkey)) {
        Ok(session) => session,
        Err(error) => {
            tracing::error!(pubkey = %mentioned_pubkey, channel, %error, "exact mention target lookup failed");
            return None;
        }
    };
    let profile_slug = state.with_store(|store| {
        store
            .get_profile(mentioned_pubkey)
            .ok()
            .flatten()
            .and_then(|profile| (!profile.agent_slug.is_empty()).then_some(profile.agent_slug))
    });
    let configured_slug = crate::identity::list_local_agent_details(&crate::config::mosaico_home())
        .into_iter()
        .find(|agent| agent.pubkey.as_deref() == Some(mentioned_pubkey))
        .map(|agent| agent.slug);
    let agent_slug = session
        .as_ref()
        .map(|session| session.agent_slug.clone())
        .or(configured_slug)
        .or_else(|| {
            active_claim
                .as_ref()
                .or(claim.as_ref())
                .map(|claim| claim.agent_slug.clone())
                .filter(|slug| !slug.is_empty())
        })
        .or(profile_slug)?;

    let addressed_member = state.with_store(|store| match session.as_ref() {
        Some(session) => store
            .is_session_joined_channel(&session.pubkey, channel)
            .unwrap_or(false),
        None => store
            .is_channel_member(channel, mentioned_pubkey)
            .unwrap_or(false),
    });
    if !addressed_member {
        tracing::warn!(
            event_id,
            pubkey = %mentioned_pubkey,
            channel,
            "exact mention target is not a channel member; refusing recovery"
        );
        return None;
    }

    let created_at = state.with_store(|store| {
        store
            .get_event(event_id)
            .ok()
            .flatten()
            .map(|event| event.created_at)
            .unwrap_or_else(now_secs)
    });
    let persisted = state.with_store(|store| {
        store.enqueue_inbox(
            event_id,
            mentioned_pubkey,
            requester_pubkey.unwrap_or_default(),
            channel,
            body,
            created_at,
        )?;
        store.add_message_recipient(event_id, mentioned_pubkey, None)?;
        Ok::<_, anyhow::Error>(())
    });
    if let Err(error) = persisted {
        tracing::error!(
            event_id,
            pubkey = %mentioned_pubkey,
            channel,
            %error,
            "exact mention could not be persisted before recovery; refusing launch"
        );
        return None;
    }
    Some(MentionTarget {
        agent_slug,
        session,
    })
}
