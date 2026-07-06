use super::*;
use crate::state::receipts::NewReceipt;

#[tokio::test]
async fn rpc_probe_validate_sub_channel_checks_h_and_d_resources() {
    let state = DaemonState::new_for_test().await;
    seed_subscriptions(&state);
    record_subscription_receipt(&state, "room");

    let v = rpc_probe(&state, &json!({ "verb": "validate", "target": "sub:room" })).unwrap();

    assert_eq!(v["ok"], true);
    assert_eq!(v["surface"], "subscriptions");
    assert_check_status(&v, "subscription_outcome", "passed");
    assert_check_status(&v, "explain", "passed");
    assert_eq!(v["subscription_evidence"]["expected_resource_count"], 2);
    assert_eq!(v["subscription_evidence"]["found_resource_count"], 2);
    assert_eq!(v["subscription_evidence"]["receipt_count"], 1);
    assert_eq!(
        v["subscription_evidence"]["resources"][0]["resource_key"],
        "sub/h/room"
    );
    assert_eq!(
        v["subscription_evidence"]["resources"][1]["resource_key"],
        "sub/d/room"
    );
}

#[tokio::test]
async fn rpc_probe_validate_sub_resource_checks_exact_pubkey_row() {
    let state = DaemonState::new_for_test().await;
    seed_subscriptions(&state);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "sub/p/pk1" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "subscription_outcome", "passed");
    assert_eq!(v["subscription_evidence"]["kind"], "resource");
    assert_eq!(v["subscription_evidence"]["expected_resource_count"], 1);
    assert_eq!(
        v["subscription_evidence"]["resources"][0]["resource_key"],
        "sub/p/pk1"
    );
}

#[tokio::test]
async fn rpc_probe_validate_sub_missing_is_not_proven_and_state_fails_target_row() {
    let state = DaemonState::new_for_test().await;
    seed_subscriptions(&state);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "sub:missing" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "subscription_outcome", "not_proven");
    assert_check_status(&v, "state", "failed");
    assert_eq!(v["subscription_evidence"]["found_resource_count"], 0);
}

fn seed_subscriptions(state: &std::sync::Arc<DaemonState>) {
    let mut sessions = BTreeMap::new();
    sessions.insert("s1".to_string(), BTreeSet::from(["room".to_string()]));
    state
        .subs
        .lock()
        .unwrap()
        .sync(&CoverageSnapshot {
            daemon_channels: BTreeSet::from(["room".to_string()]),
            addressed_pubkeys: BTreeSet::from(["pk1".to_string()]),
            archived_channels: BTreeSet::new(),
            sessions,
        })
        .unwrap();
}

fn record_subscription_receipt(state: &std::sync::Arc<DaemonState>, channel: &str) {
    state
        .with_store(|s| {
            s.record_receipt(&NewReceipt {
                surface: "subscriptions".into(),
                transaction_id: 1,
                revision: 1,
                changed_summary: format!(r#"{{"channel":"{channel}"}}"#),
                commands: format!(r#"[{{"key":"sub/h/{channel}"}}]"#),
                artifact_ref: None,
                created_at: 100,
            })?;
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();
}

fn assert_check_status(v: &serde_json::Value, name: &str, status: &str) {
    assert_eq!(check_row(v, name)["status"], status);
}

fn check_row<'a>(v: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    v["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .expect("check row")
}
