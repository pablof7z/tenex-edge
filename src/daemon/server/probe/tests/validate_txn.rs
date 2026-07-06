use super::*;
use crate::state::receipts::NewReceipt;
use crate::state::trellis_commits::NewCommit;

#[tokio::test]
async fn rpc_probe_validate_txn_target_matches_commit_and_receipt() {
    let state = DaemonState::new_for_test().await;
    record_commit(&state, "status", 7, 3);
    record_receipt(&state, "status", 7, 3);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "txn:status:7" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_eq!(v["surface"], "status");
    assert_check_status(&v, "txn_outcome", "passed");
    assert_check_status(&v, "explain", "passed");
    assert_eq!(v["txn_evidence"]["commit_count"], 1);
    assert_eq!(v["txn_evidence"]["receipt_count"], 1);
    assert_eq!(v["txn_evidence"]["receipt_revisions_match_commits"], true);
}

#[tokio::test]
async fn rpc_probe_validate_txn_target_timestamp_disambiguates_epochs() {
    let state = DaemonState::new_for_test().await;
    record_commit_at(&state, "status", 7, 3, 100);
    record_receipt_at(&state, "status", 7, 3, 101);
    record_commit_at(&state, "status", 7, 10, 500);
    record_receipt_at(&state, "status", 7, 10, 501);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "txn:status:7@500" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "txn_outcome", "passed");
    assert_check_status(&v, "explain", "passed");
    assert_eq!(v["txn_evidence"]["total_commit_count"], 2);
    assert_eq!(v["txn_evidence"]["commit_count"], 1);
    assert_eq!(v["txn_evidence"]["latest_commit"]["revision"], 10);
    assert_eq!(v["txn_evidence"]["receipts"][0]["revision"], 10);
    assert_eq!(v["explain"]["receipts"][0]["revision"], 10);
}

#[tokio::test]
async fn rpc_probe_validate_txn_target_accepts_commit_without_effect_receipt() {
    let state = DaemonState::new_for_test().await;
    record_commit(&state, "cursor", 8, 4);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "txn:cursor:8" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_eq!(v["verdict"], "passed_with_limitations");
    assert_check_status(&v, "txn_outcome", "passed");
    assert_check_status(&v, "explain", "not_proven");
    assert_eq!(v["txn_evidence"]["commit_count"], 1);
    assert_eq!(v["txn_evidence"]["receipt_count"], 0);
    assert!(v["limitations"].as_array().unwrap().iter().any(|l| l
        .as_str()
        .unwrap()
        .contains("commit ledger is the explanation")));
}

#[tokio::test]
async fn rpc_probe_validate_txn_target_fails_receipt_without_commit() {
    let state = DaemonState::new_for_test().await;
    record_receipt(&state, "status", 9, 5);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "txn:status:9" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "txn_outcome", "failed");
    assert_eq!(v["txn_evidence"]["commit_count"], 0);
    assert_eq!(v["txn_evidence"]["receipt_count"], 1);
}

fn record_commit(
    state: &std::sync::Arc<DaemonState>,
    surface: &str,
    transaction_id: i64,
    revision: i64,
) {
    record_commit_at(state, surface, transaction_id, revision, 100);
}

fn record_commit_at(
    state: &std::sync::Arc<DaemonState>,
    surface: &str,
    transaction_id: i64,
    revision: i64,
    created_at: i64,
) {
    state
        .with_store(|s| {
            s.record_commit(&NewCommit {
                surface: surface.into(),
                transaction_id,
                revision,
                mode: "drive".into(),
                trigger_kind: "test".into(),
                trigger_ref: "fixture".into(),
                changed_inputs_json: r#"["input"]"#.into(),
                changed_derived_json: "[]".into(),
                changed_collections_json: "[]".into(),
                resource_commands_json: "[]".into(),
                output_frames_json: "[]".into(),
                command_count: 0,
                output_count: 0,
                effect_count: 0,
                suppressed_count: 0,
                noop: 0,
                oracle_status: Some("green".into()),
                oracle_error: None,
                duration_us: 10,
                graph_nodes: 3,
                graph_resources: 1,
                created_at,
            })?;
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();
}

fn record_receipt(
    state: &std::sync::Arc<DaemonState>,
    surface: &str,
    transaction_id: i64,
    revision: i64,
) {
    record_receipt_at(state, surface, transaction_id, revision, 101);
}

fn record_receipt_at(
    state: &std::sync::Arc<DaemonState>,
    surface: &str,
    transaction_id: i64,
    revision: i64,
    created_at: i64,
) {
    state
        .with_store(|s| {
            s.record_receipt(&NewReceipt {
                surface: surface.into(),
                transaction_id,
                revision,
                changed_summary: r#"{"changed":true}"#.into(),
                commands: "[]".into(),
                artifact_ref: Some(format!("evt-{surface}-{transaction_id}")),
                created_at,
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
