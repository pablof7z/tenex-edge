use crate::state::{RecordMessage, RelayEvent, Store};

pub(in crate::daemon::server) struct ChatSeed<'a> {
    pub event_id: &'a str,
    pub from_pubkey: &'a str,
    pub from_session: Option<&'a str>,
    pub channel_h: &'a str,
    pub body: &'a str,
    pub mentioned_pubkey: Option<&'a str>,
    pub mentioned_session: Option<&'a str>,
    pub created_at: u64,
    pub direction: &'a str,
}

/// Build a verbatim kind:9 chat row for the `relay_events` log from the fields
/// we already know about a freshly-published chat line.
pub(in crate::daemon::server) fn chat_relay_event(seed: &ChatSeed<'_>) -> RelayEvent {
    let mut tags: Vec<Vec<String>> = vec![vec!["h".to_string(), seed.channel_h.to_string()]];
    if let Some(pk) = seed.mentioned_pubkey {
        tags.push(vec!["p".to_string(), pk.to_string()]);
    }
    RelayEvent {
        id: seed.event_id.to_string(),
        kind: crate::fabric::nip29::wire::KIND_CHAT as u32,
        pubkey: seed.from_pubkey.to_string(),
        created_at: seed.created_at,
        channel_h: seed.channel_h.to_string(),
        d_tag: String::new(),
        content: seed.body.to_string(),
        tags_json: serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string()),
    }
}

pub(in crate::daemon::server) fn seed_chat_read_models(
    store: &Store,
    seed: &ChatSeed<'_>,
    context: &str,
) {
    if let Err(e) = store.insert_event(&chat_relay_event(seed)) {
        tracing::error!(
            event_id = %seed.event_id,
            channel = %seed.channel_h,
            error = %e,
            "{context}: seeding chat into relay_events failed"
        );
    }
    if let Err(e) = store.record_message(&RecordMessage {
        message_id: seed.event_id.to_string(),
        thread_id: seed.channel_h.to_string(),
        channel_h: seed.channel_h.to_string(),
        author_pubkey: seed.from_pubkey.to_string(),
        author_session: seed.from_session.map(str::to_string),
        body: seed.body.to_string(),
        created_at: seed.created_at,
        direction: seed.direction.to_string(),
        sync_state: "accepted".to_string(),
        native_event_id: Some(seed.event_id.to_string()),
        error: None,
    }) {
        tracing::error!(
            event_id = %seed.event_id,
            channel = %seed.channel_h,
            error = %e,
            "{context}: seeding chat into messages failed"
        );
    }
    if let Some(pk) = seed.mentioned_pubkey {
        if let Err(e) = store.add_message_recipient(seed.event_id, pk, seed.mentioned_session, None)
        {
            tracing::error!(
                event_id = %seed.event_id,
                channel = %seed.channel_h,
                error = %e,
                "{context}: seeding message recipient failed"
            );
        }
    }
}
