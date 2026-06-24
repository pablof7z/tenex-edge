use super::*;
use crate::util::session_codename;

#[test]
fn who_snapshot_uses_session_pubkey_for_transient_local_session() {
    let store = Store::open_memory().unwrap();
    let id = register_local(
        &store,
        "claude",
        "pk-claude",
        "proj",
        "laptop",
        "",
        "sid-transient",
        1_000,
    );
    store
        .upsert_session_pubkey("pk-session", &id, "pk-claude", "claude", 1_000)
        .unwrap();
    record_peer(
        &store,
        "pk-session",
        "bravo123 (claude)",
        "proj",
        "remote-echo",
        "laptop",
        "",
        "",
        false,
        1_000,
    );

    let snapshot = load_who_snapshot(&store, Some("proj"), 1_000, "laptop").unwrap();

    assert_eq!(
        snapshot.rows.len(),
        1,
        "session-key peer echo must be hidden"
    );
    let row = snapshot.rows.first().unwrap();
    assert_eq!(row.source, WhoSource::Local);
    assert_eq!(row.slug, format!("{} (claude)", session_codename(&id)));
    assert_eq!(row.pubkey, "pk-session");

    let rendered = strip_ansi(&render_who_plain(&snapshot));
    assert!(rendered.contains(&format!("{} (claude)", session_codename(&id))));
    assert!(
        !rendered.contains(&format!("claude-{}", session_codename(&id))),
        "transient local identity must not use durable duplicate disambiguation: {rendered}"
    );
}

#[test]
fn channel_status_map_keys_local_transient_session_by_session_pubkey() {
    let store = Store::open_memory().unwrap();
    let id = register_local(
        &store,
        "claude",
        "pk-claude",
        "proj",
        "laptop",
        "",
        "sid-transient",
        1_000,
    );
    store
        .upsert_session_pubkey("pk-session", &id, "pk-claude", "claude", 1_000)
        .unwrap();

    let map = channel_status_map(&store, "proj", 1_000);

    assert!(map.contains_key("pk-session"));
    assert!(!map.contains_key("pk-claude"));
}
