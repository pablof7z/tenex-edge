use crate::daemon_harness::shared_nip29_relay_url;
use mosaico::fabric::nip29::wire::KIND_PROFILE;
use nostr_sdk::prelude::{Client as NostrClient, ClientOptions, EventBuilder, Filter, Keys, Kind};
use nostr_sdk::NostrSigner;

pub(super) async fn publish_profile(keys: &Keys, name: &str, agent_slug: &str) {
    let client = NostrClient::builder()
        .signer(keys.clone())
        .opts(ClientOptions::default().automatic_authentication(true))
        .build();
    client
        .add_relay(shared_nip29_relay_url())
        .await
        .expect("add relay");
    client.connect().await;
    client
        .wait_for_connection(std::time::Duration::from_secs(8))
        .await;
    let _ = client
        .fetch_events(
            Filter::new().kind(Kind::from(0u16)).limit(1),
            std::time::Duration::from_secs(5),
        )
        .await;
    let builder = EventBuilder::new(
        Kind::from(KIND_PROFILE),
        serde_json::json!({ "name": name }).to_string(),
    )
    .tags([nostr_sdk::Tag::parse(["agent-slug", agent_slug]).unwrap()]);
    let unsigned = builder.build(keys.public_key());
    let signed = keys.sign_event(unsigned).await.expect("sign profile");
    let out = client.send_event(&signed).await.expect("publish profile");
    assert!(
        !out.success.is_empty(),
        "profile publish rejected: success={:?} failed={:?}",
        out.success,
        out.failed
    );
}
