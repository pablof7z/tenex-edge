use super::*;

fn record(id: &str, author_session: Option<&str>, direction: &str) -> RecordMessage {
    record_at(id, author_session, direction, "accepted", 10)
}

fn record_at(
    id: &str,
    author_session: Option<&str>,
    direction: &str,
    sync_state: &str,
    created_at: u64,
) -> RecordMessage {
    RecordMessage {
        message_id: id.to_string(),
        thread_id: "chan".to_string(),
        channel_h: "chan".to_string(),
        author_pubkey: "author-pk".to_string(),
        author_session: author_session.map(str::to_string),
        body: "hello".to_string(),
        created_at,
        direction: direction.to_string(),
        sync_state: sync_state.to_string(),
        native_event_id: Some(id.to_string()),
        error: None,
    }
}

fn register_sender_session(store: &Store) {
    store
        .upsert_session_row(
            "sender-session",
            &RegisterSession {
                harness: "codex".to_string(),
                external_id_kind: "pid".to_string(),
                external_id: "123".to_string(),
                agent_pubkey: "author-pk".to_string(),
                agent_slug: "codex".to_string(),
                channel_h: "chan".to_string(),
                child_pid: Some(123),
                transcript_path: None,
                resume_id: "native-session".to_string(),
                now: 1,
            },
        )
        .unwrap();
}

#[test]
fn replay_without_session_does_not_erase_return_envelope() {
    let store = Store::open_memory().unwrap();
    store
        .record_message(&record("event-1", Some("sender-session"), "outbound"))
        .unwrap();
    store
        .record_message(&record("event-1", None, "inbound"))
        .unwrap();

    let msg = store.get_message("event-1").unwrap().unwrap();
    assert_eq!(msg.author_session.as_deref(), Some("sender-session"));
    assert_eq!(msg.direction, "outbound");
}

#[test]
fn relay_event_backfill_derives_author_session_from_status() {
    let store = Store::open_memory().unwrap();
    store
        .upsert_status(&Status {
            pubkey: "author-pk".to_string(),
            session_id: "sender-session".to_string(),
            channel_h: "chan".to_string(),
            slug: "writer".to_string(),
            title: String::new(),
            activity: String::new(),
            busy: false,
            last_seen: 9,
            updated_at: 9,
            expiration: 99,
        })
        .unwrap();
    store
        .insert_event(&RelayEvent {
            id: "event-2".to_string(),
            kind: crate::fabric::nip29::wire::KIND_CHAT as u32,
            pubkey: "author-pk".to_string(),
            created_at: 10,
            channel_h: "chan".to_string(),
            d_tag: String::new(),
            content: "from relay".to_string(),
            tags_json: "[]".to_string(),
        })
        .unwrap();

    store.backfill_messages_from_relay_events().unwrap();

    let msg = store.get_message("event-2").unwrap().unwrap();
    assert_eq!(msg.author_session.as_deref(), Some("sender-session"));
    assert_eq!(msg.body, "from relay");
}

#[test]
fn outbound_reply_check_only_counts_visible_session_chat_since_cutoff() {
    let store = Store::open_memory().unwrap();
    register_sender_session(&store);
    store
        .record_message(&record_at(
            "old-outbound",
            Some("sender-session"),
            "outbound",
            "accepted",
            99,
        ))
        .unwrap();
    store
        .record_message(&record_at(
            "inbound",
            Some("sender-session"),
            "inbound",
            "accepted",
            101,
        ))
        .unwrap();
    store
        .record_message(&record_at(
            "failed-outbound",
            Some("sender-session"),
            "outbound",
            "failed",
            102,
        ))
        .unwrap();

    assert!(!store
        .session_has_outbound_message_since("sender-session", 100)
        .unwrap());

    store
        .record_message(&record_at(
            "accepted-outbound",
            Some("sender-session"),
            "outbound",
            "accepted",
            100,
        ))
        .unwrap();

    assert!(store
        .session_has_outbound_message_since("sender-session", 100)
        .unwrap());
    assert!(!store
        .session_has_outbound_message_since("other-session", 100)
        .unwrap());
}
