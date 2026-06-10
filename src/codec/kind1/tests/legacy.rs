use super::*;

#[test]
fn kind_24011_presence_is_ignored() {
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::from(24011u16), "")
        .tags([
            tag(&["h", "tenex-edge"]).unwrap(),
            tag(&["session-id", "sess-123"]).unwrap(),
            tag(&["agent", &keys.public_key().to_hex(), "coder"]).unwrap(),
            tag(&["expiration", "1900000000"]).unwrap(),
        ])
        .sign_with_keys(&keys)
        .unwrap();
    assert!(Kind1Codec.decode(&event).is_none());
}

#[test]
fn t_only_project_notes_are_ignored() {
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::from(KIND_NOTE), "old shape")
        .tags([
            tag(&["t", "tenex-edge"]).unwrap(),
            tag(&["agent", &keys.public_key().to_hex(), "coder"]).unwrap(),
        ])
        .sign_with_keys(&keys)
        .unwrap();
    assert!(Kind1Codec.decode(&event).is_none());
}
