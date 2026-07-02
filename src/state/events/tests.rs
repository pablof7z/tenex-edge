use super::*;

fn event(id: &str, created_at: u64) -> RelayEvent {
    RelayEvent {
        id: id.into(),
        kind: crate::fabric::nip29::wire::KIND_CHAT as u32,
        pubkey: "pk".into(),
        created_at,
        channel_h: "h1".into(),
        d_tag: String::new(),
        content: String::new(),
        tags_json: "[]".into(),
    }
}

#[test]
fn chat_for_channel_after_preserves_same_second_id_cursor() {
    let store = Store::open_memory().unwrap();
    assert!(store.insert_event(&event("a", 10)).unwrap());
    assert!(store.insert_event(&event("b", 10)).unwrap());
    assert!(store.insert_event(&event("c", 11)).unwrap());

    let rows = store.chat_for_channel_after("h1", 10, "a", 10).unwrap();
    assert_eq!(
        rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        vec!["b", "c"]
    );

    let rows = store.chat_for_channel_after("h1", 10, "b", 10).unwrap();
    assert_eq!(
        rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        vec!["c"]
    );
}
