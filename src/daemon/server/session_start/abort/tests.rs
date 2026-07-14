use super::*;

#[tokio::test]
async fn dropping_guard_after_claim_releases_claim_and_identity() {
    let state = DaemonState::new_for_test().await;
    let keys = nostr_sdk::prelude::Keys::generate();
    let pubkey = keys.public_key().to_hex();
    let agent = crate::identity::AgentIdentity {
        slug: "chief".into(),
        keys,
        commands: Vec::new(),
        per_session_key: false,
        harness: None,
    };
    let minted = mint_session_identity(
        &state,
        "orphan",
        &agent,
        "root",
        SessionIdentityInput::new("", None),
        None,
    )
    .unwrap();
    assert!(minted.durable_claim_acquired);
    drop(SessionStartGuard::new(&state, &minted, false));

    state.with_store(|s| {
        assert!(s
            .live_durable_session_for_pubkey(&pubkey)
            .unwrap()
            .is_none());
        assert!(s
            .get_identity(&pubkey)
            .unwrap()
            .is_none_or(|identity| !identity.alive));
    });
}

#[tokio::test]
async fn reassert_guard_never_rolls_back_the_existing_normal_identity() {
    let state = DaemonState::new_for_test().await;
    let agent = crate::identity::AgentIdentity {
        slug: "codex".into(),
        keys: nostr_sdk::prelude::Keys::generate(),
        commands: Vec::new(),
        per_session_key: true,
        harness: None,
    };
    let minted = mint_session_identity(
        &state,
        "running",
        &agent,
        "root",
        SessionIdentityInput::new("native", None),
        None,
    )
    .unwrap();
    let pubkey = minted.identity.pubkey.clone();

    drop(SessionStartGuard::new(&state, &minted, true));

    let identity = state
        .with_store(|store| store.get_identity(&pubkey))
        .unwrap()
        .expect("existing identity remains");
    assert!(identity.alive);
    assert!(state
        .with_store(|store| store.handle_for_pubkey(&pubkey))
        .unwrap()
        .is_some());
}
