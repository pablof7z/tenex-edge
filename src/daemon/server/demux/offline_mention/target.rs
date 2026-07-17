use crate::daemon::server::DaemonState;
use crate::util::now_secs;
use std::sync::Arc;

pub(super) struct MentionTarget {
    pub(super) agent_slug: String,
    pub(super) session: Option<crate::state::Session>,
}

pub(super) enum Resolution {
    Ready(MentionTarget),
    Retry,
    Reject,
}

pub(super) fn resolve_and_persist(
    state: &Arc<DaemonState>,
    event_id: &str,
    mentioned_pubkey: &str,
    channel: &str,
    body: &str,
    requester_pubkey: Option<&str>,
) -> Resolution {
    let session = match state.with_store(|store| store.get_session(mentioned_pubkey)) {
        Ok(session) => session,
        Err(error) => {
            tracing::error!(pubkey = %mentioned_pubkey, channel, %error, "exact mention target lookup failed");
            return Resolution::Retry;
        }
    };
    let configured_slug = crate::identity::list_local_agent_details(&crate::config::mosaico_home())
        .into_iter()
        .find(|agent| agent.pubkey.as_deref() == Some(mentioned_pubkey))
        .map(|agent| agent.slug);
    let Some(agent_slug) = session
        .as_ref()
        .map(|session| session.agent_slug.clone())
        .or(configured_slug)
    else {
        tracing::warn!(
            event_id,
            pubkey = %mentioned_pubkey,
            channel,
            "exact mention target has no locally owned session or configured identity"
        );
        return Resolution::Reject;
    };

    // A stopped session may no longer be a relay member after its retention
    // window. The durable local channel affinity remains the authorization to
    // resume that exact pubkey; current sender membership was already enforced
    // by fabric admission.
    let addressed_affinity = state.with_store(|store| match session.as_ref() {
        Some(session) => store
            .has_session_route(&session.pubkey, channel)
            .unwrap_or(false),
        None => store
            .is_channel_member(channel, mentioned_pubkey)
            .unwrap_or(false),
    });
    if !addressed_affinity {
        tracing::warn!(
            event_id,
            pubkey = %mentioned_pubkey,
            channel,
            "exact mention target has no durable channel affinity; refusing recovery"
        );
        return Resolution::Reject;
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
        return Resolution::Retry;
    }
    Resolution::Ready(MentionTarget {
        agent_slug,
        session,
    })
}
