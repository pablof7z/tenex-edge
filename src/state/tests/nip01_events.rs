use super::super::*;

#[test]
fn nip01_replaceable_replaces_by_kind_pubkey() {
    let s = Store::open_memory().unwrap();
    let mut ev = RelayEvent {
        id: "e1".into(),
        kind: 10002,
        pubkey: "pk".into(),
        created_at: 100,
        channel_h: String::new(),
        d_tag: String::new(),
        content: "old".into(),
        tags_json: "[]".into(),
    };
    assert!(s.insert_event(&ev).unwrap());
    ev.id = "e2".into();
    ev.created_at = 200;
    ev.content = "new".into();
    assert!(s.insert_event(&ev).unwrap());
    assert!(s.get_event("e1").unwrap().is_none());
    assert_eq!(s.get_event("e2").unwrap().unwrap().content, "new");
    // An older event loses the race and is not stored.
    ev.id = "e0".into();
    ev.created_at = 50;
    assert!(!s.insert_event(&ev).unwrap());
    assert!(s.get_event("e0").unwrap().is_none());
}

#[test]
fn nip01_addressable_replaces_by_kind_pubkey_dtag() {
    let s = Store::open_memory().unwrap();
    let mk = |id: &str, ts: u64, d: &str| RelayEvent {
        id: id.into(),
        kind: 30078,
        pubkey: "pk".into(),
        created_at: ts,
        channel_h: String::new(),
        d_tag: d.into(),
        content: String::new(),
        tags_json: "[]".into(),
    };
    assert!(s.insert_event(&mk("a", 1, "d1")).unwrap());
    assert!(s.insert_event(&mk("b", 1, "d2")).unwrap());
    // Replace d1 only; d2 survives (different coordinate).
    assert!(s.insert_event(&mk("c", 2, "d1")).unwrap());
    assert!(s.get_event("a").unwrap().is_none());
    assert!(s.get_event("b").unwrap().is_some());
    assert!(s.get_event("c").unwrap().is_some());
}

#[test]
fn nip01_regular_appends() {
    let s = Store::open_memory().unwrap();
    let mk = |id: &str| RelayEvent {
        id: id.into(),
        kind: 1,
        pubkey: "pk".into(),
        created_at: 1,
        channel_h: "h1".into(),
        d_tag: String::new(),
        content: String::new(),
        tags_json: "[]".into(),
    };
    assert!(s.insert_event(&mk("n1")).unwrap());
    assert!(s.insert_event(&mk("n2")).unwrap());
    assert_eq!(s.chat_for_channel("h1", 0, 10).unwrap().len(), 2);
}
