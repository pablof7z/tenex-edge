use crate::state::{Session, Status, Store};

pub(super) const BACKEND: &str = "pk-backend";

/// Publish a relay_status (kind:30315) row — the single source awareness reads
/// for "who is doing what here", local and remote alike.
#[allow(clippy::too_many_arguments)]
pub(super) fn pub_status(
    store: &Store,
    pubkey: &str,
    slug: &str,
    title: &str,
    activity: &str,
    busy: bool,
    updated_at: u64,
    now: u64,
) {
    store
        .upsert_status(&Status {
            pubkey: pubkey.to_string(),
            channel_h: "proj".to_string(),
            slug: slug.to_string(),
            title: title.to_string(),
            activity: activity.to_string(),
            busy,
            last_seen: updated_at,
            updated_at,
            expiration: now + 90,
        })
        .unwrap();
}

/// Materialize the `proj` channel + roster so awareness has fabric context.
pub(super) fn seed_channel(store: &Store) {
    // Opaque id "proj" with a distinct human name "main" (production ids are random, never the name).
    store.upsert_channel("proj", "main", "", "", 1).unwrap();
    store
        .replace_channel_members("proj", &["pk-coder".to_string()], 1)
        .unwrap();
    store
        .upsert_profile_with_agent_slug("pk-coder", "coder", "coder", "coder", "laptop", false, 1)
        .unwrap();
}

pub(super) fn test_session(_id: &str) -> Session {
    Session {
        pubkey: "pk-coder".to_string(),
        runtime_generation: 1,
        agent_slug: "coder".to_string(),
        channel_h: "proj".to_string(),
        harness: "claude-code".to_string(),
        child_pid: None,
        transcript_path: None,
        alive: true,
        created_at: 1,
        last_seen: 1,
        working: false,
        turn_started_at: 0,
        last_distill_at: 0,
        work_topic: String::new(),
        work_topic_set_at: 0,
        seen_cursor: 0,
        title: String::new(),
        activity: String::new(),
        distill_fail_streak: 0,
        distill_notice_at: 0,
        explicit_chat_published_at: 0,
    }
}
