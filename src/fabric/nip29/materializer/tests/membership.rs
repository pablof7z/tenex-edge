use super::*;

#[test]
fn older_member_roster_materialization_does_not_replace_newer_roster() {
    let store = Store::open_memory().unwrap();
    let relay = Keys::generate();
    let old_member = Keys::generate().public_key().to_hex();
    let new_member = Keys::generate().public_key().to_hex();

    let newer = build_at(
        &relay,
        39002,
        "",
        vec![make_tag(&["d", "proj"]), make_tag(&["p", &new_member])],
        20,
    );
    let older = build_at(
        &relay,
        39002,
        "",
        vec![make_tag(&["d", "proj"]), make_tag(&["p", &old_member])],
        10,
    );
    Nip29Materializer::materialize_members(&store, &newer);
    Nip29Materializer::materialize_members(&store, &older);

    assert!(store.is_channel_member("proj", &new_member).unwrap());
    assert!(!store.is_channel_member("proj", &old_member).unwrap());
}

#[test]
fn older_admin_roster_materialization_does_not_replace_newer_roster() {
    let store = Store::open_memory().unwrap();
    let relay = Keys::generate();
    let old_admin = Keys::generate().public_key().to_hex();
    let new_admin = Keys::generate().public_key().to_hex();

    let newer = build_at(
        &relay,
        39001,
        "",
        vec![make_tag(&["d", "proj"]), make_tag(&["p", &new_admin])],
        20,
    );
    let older = build_at(
        &relay,
        39001,
        "",
        vec![make_tag(&["d", "proj"]), make_tag(&["p", &old_admin])],
        10,
    );
    Nip29Materializer::materialize_admins(&store, &newer);
    Nip29Materializer::materialize_admins(&store, &older);

    assert!(store.is_channel_admin("proj", &new_admin).unwrap());
    assert!(!store.is_channel_admin("proj", &old_admin).unwrap());
}
