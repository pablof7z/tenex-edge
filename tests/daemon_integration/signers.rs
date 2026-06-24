use crate::daemon_harness::*;
use std::time::{SystemTime, UNIX_EPOCH};
use tenex_edge::daemon::client::Client;
use tenex_edge::state::Store;

#[path = "signers/relay.rs"]
mod relay;

const EXAMPLE_USER_NSEC: &str = "nsec1eulru7a67wt9ndqxv424kmgvd6uyd8defdxh7y9peut28f2p2vhs35m5h4";
const EXAMPLE_BACKEND_SEC_HEX: &str =
    "b53809614e9c41b923dd5546e438e48e9bcbee04b9c7c50bae0b085954e03422";

fn pubkey_of(sec: &str) -> String {
    use nostr_sdk::prelude::Keys;
    Keys::parse(sec).unwrap().public_key().to_hex()
}

fn rewrite_config_with_signing_relay(home: &Home) {
    let _ = rewrite_config_with_nak_relay(home);
}

fn rewrite_config_with_nak_relay(home: &Home) -> String {
    let relay = shared_relay_url();
    let user_pk = pubkey_of(EXAMPLE_USER_NSEC);
    let cfg = home.dir.path().join("config.json");
    let body = serde_json::json!({
        "whitelistedPubkeys": [user_pk],
        "backendName": "test-host",
        "relays": [relay],
        "indexerRelay": relay,
        "userNsec": EXAMPLE_USER_NSEC,
        "tenexPrivateKey": EXAMPLE_BACKEND_SEC_HEX,
    });
    std::fs::write(&cfg, serde_json::to_string(&body).unwrap()).unwrap();
    relay
}

fn unique_channel(label: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("issue22-{label}-{nanos}")
}

async fn start_session(
    client: &mut Client,
    agent: &str,
    harness_id: Option<&str>,
    resume_id: Option<&str>,
    channel: &str,
) -> String {
    let mut params = serde_json::json!({
        "agent": agent,
        "cwd": "/tmp",
        "channel": channel,
        "harness": "codex",
    });
    if let Some(id) = harness_id {
        params["session_id"] = serde_json::json!(id);
    }
    if let Some(id) = resume_id {
        params["resume_id"] = serde_json::json!(id);
    }
    let v = client
        .call("session_start", params)
        .await
        .expect("session_start");
    v["session_id"].as_str().unwrap().to_string()
}

fn assert_who_lists_claude(home: &Home) {
    let out = run_cli(home, &["who", "--all-projects"]);
    assert!(
        out.status.success(),
        "who failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("(claude)"),
        "who should display codename rows with the claude agent: {stdout}"
    );
    eprintln!("who evidence:\n{stdout}");
}

#[test]
fn duplicate_same_agent_same_channel_gets_transient_signer() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_signing_relay(&home);
    let channel = unique_channel("collision");
    let other_channel = unique_channel("other");

    let (first_id, second_id, other_id) = rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        let first = start_session(&mut c, "claude", Some("issue22-first"), None, &channel).await;
        let second = start_session(&mut c, "claude", Some("issue22-second"), None, &channel).await;
        let other = start_session(
            &mut c,
            "claude",
            Some("issue22-other"),
            None,
            &other_channel,
        )
        .await;
        (first, second, other)
    });

    let store = Store::open(&home.store_path()).unwrap();
    let durable_pubkey = store
        .get_session(&first_id)
        .unwrap()
        .expect("first session")
        .agent_pubkey;
    assert!(
        store.session_pubkey_for_session(&first_id).is_none(),
        "first same-agent/channel session should keep durable signer"
    );
    let transient_pubkey = store
        .session_pubkey_for_session(&second_id)
        .expect("second same-agent/channel session should get transient signer");
    assert_ne!(
        transient_pubkey, durable_pubkey,
        "transient signer must differ from durable agent pubkey"
    );
    assert!(
        store.session_pubkey_for_session(&other_id).is_none(),
        "same durable agent in a different channel should keep durable signer"
    );

    stop_daemon(&home);
}

#[test]
fn nak_relay_observes_transient_duplicate_status_author() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    let relay = rewrite_config_with_nak_relay(&home);
    let channel = unique_channel("nak-hello");
    let other_channel = unique_channel("nak-backend");

    let (first_id, second_id, other_id, whoami_pubkey) = rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        let first =
            start_session(&mut c, "claude", Some("issue22-nak-first"), None, &channel).await;
        let second =
            start_session(&mut c, "claude", Some("issue22-nak-second"), None, &channel).await;
        let other = start_session(
            &mut c,
            "claude",
            Some("issue22-nak-other"),
            None,
            &other_channel,
        )
        .await;
        let whoami = c
            .call("whoami", serde_json::json!({"session": second.clone()}))
            .await
            .expect("whoami");
        (
            first,
            second,
            other,
            whoami["pubkey"].as_str().unwrap().to_string(),
        )
    });

    let store = Store::open(&home.store_path()).unwrap();
    let durable_pubkey = store
        .get_session(&first_id)
        .unwrap()
        .expect("first session")
        .agent_pubkey;
    assert!(store.session_pubkey_for_session(&first_id).is_none());
    assert!(store.session_pubkey_for_session(&other_id).is_none());
    let transient_pubkey = store
        .session_pubkey_for_session(&second_id)
        .expect("duplicate session should use transient pubkey");
    assert_eq!(
        whoami_pubkey, transient_pubkey,
        "whoami should report the transient signer for the duplicate"
    );

    assert!(
        wait_until(std::time::Duration::from_secs(20), || {
            relay::relay_has_status_authors(
                &relay,
                &channel,
                &[durable_pubkey.as_str(), transient_pubkey.as_str()],
            )
        }),
        "nak serve should contain durable and transient kind:30315 authors in {channel}; got {:?}",
        relay::status_authors_on_relay(&relay, &channel)
    );
    eprintln!(
        "nak serve relay={relay} channel={channel} status evidence={:?}",
        relay::status_evidence_on_relay(&relay, &channel)
    );
    assert!(
        wait_until(std::time::Duration::from_secs(20), || {
            relay::relay_has_status_authors(&relay, &other_channel, &[durable_pubkey.as_str()])
        }),
        "nak serve should contain durable kind:30315 author in {other_channel}; got {:?}",
        relay::status_authors_on_relay(&relay, &other_channel)
    );
    eprintln!(
        "nak serve relay={relay} channel={other_channel} status evidence={:?}",
        relay::status_evidence_on_relay(&relay, &other_channel)
    );
    eprintln!("whoami duplicate pubkey={whoami_pubkey}");

    assert_who_lists_claude(&home);
    stop_daemon(&home);
}

#[test]
fn duplicate_resume_reassert_preserves_transient_pubkey() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_signing_relay(&home);
    let channel = unique_channel("resume");

    let (duplicate_id, whoami_pubkey, whoami_npub) = rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        start_session(&mut c, "claude", Some("issue22-anchor-a"), None, &channel).await;
        let duplicate = start_session(
            &mut c,
            "claude",
            None,
            Some("issue22-resume-token"),
            &channel,
        )
        .await;
        let whoami = c
            .call("whoami", serde_json::json!({"session": duplicate.clone()}))
            .await
            .expect("whoami");
        (
            duplicate,
            whoami["pubkey"].as_str().unwrap().to_string(),
            whoami["npub"].as_str().unwrap().to_string(),
        )
    });
    let before = Store::open(&home.store_path())
        .unwrap()
        .session_pubkey_for_session(&duplicate_id)
        .expect("duplicate session should have transient pubkey");
    let expected_npub = {
        use nostr_sdk::prelude::ToBech32;
        nostr_sdk::PublicKey::from_hex(&before)
            .unwrap()
            .to_bech32()
            .unwrap()
    };
    assert_eq!(
        whoami_pubkey, before,
        "whoami should expose selected signer"
    );
    assert_eq!(
        whoami_npub, expected_npub,
        "whoami npub should match selected signer"
    );

    let duplicate_id_after = rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        start_session(
            &mut c,
            "claude",
            None,
            Some("issue22-resume-token"),
            &channel,
        )
        .await
    });

    let store = Store::open(&home.store_path()).unwrap();
    let after = store
        .session_pubkey_for_session(&duplicate_id_after)
        .expect("reasserted duplicate should keep transient pubkey");
    assert_eq!(
        duplicate_id, duplicate_id_after,
        "resume_id should resolve to the same canonical session"
    );
    assert_eq!(before, after, "resume must preserve transient pubkey");

    stop_daemon(&home);
}
