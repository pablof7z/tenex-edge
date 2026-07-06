use super::*;
use crate::state::receipts::NewReceipt;
use crate::state::trellis_commits::NewCommit;

#[tokio::test]
async fn rpc_probe_validate_commit_target_reports_durable_commit_row() {
    let state = DaemonState::new_for_test().await;
    let id = record_commit(&state, commit_fixture("status", 7, 3));
    record_receipt(&state, "status", 7, 3);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("commit:{id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "commit_outcome", "passed");
    assert_eq!(v["commit_evidence"]["commit_id"], id);
    assert_eq!(v["commit_evidence"]["surface"], "status");
    assert_eq!(v["commit_evidence"]["matching_receipt_count"], 1);
    assert_eq!(v["commit_evidence"]["receipt_delta_ms"], 1);
    assert_eq!(v["commit_evidence"]["payload_valid"], true);
    assert_eq!(v["commit_evidence"]["command_count_matches"], true);
}

#[tokio::test]
async fn rpc_probe_validate_commit_target_accepts_receiptless_noop_commit() {
    let state = DaemonState::new_for_test().await;
    let mut commit = commit_fixture("cursor", 8, 4);
    commit.noop = 1;
    commit.suppressed_count = 1;
    let id = record_commit(&state, commit);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("trellis_commit:{id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_eq!(v["verdict"], "passed_with_limitations");
    assert_check_status(&v, "commit_outcome", "passed");
    assert_eq!(v["commit_evidence"]["noop"], true);
    assert!(v["limitations"].as_array().unwrap().iter().any(|l| {
        l.as_str()
            .unwrap()
            .contains("all-commit ledger is the explanation")
    }));
}

#[tokio::test]
async fn rpc_probe_validate_commit_target_fails_count_mismatch() {
    let state = DaemonState::new_for_test().await;
    let mut commit = commit_fixture("status", 9, 5);
    commit.command_count = 2;
    let id = record_commit(&state, commit);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("commit/{id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "commit_outcome", "failed");
    assert_eq!(v["commit_evidence"]["command_json_count"], 1);
    assert_eq!(v["commit_evidence"]["command_count"], 2);
    assert_eq!(v["commit_evidence"]["command_count_matches"], false);
}

#[tokio::test]
async fn rpc_probe_validate_missing_commit_is_not_proven() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "commit:404" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "commit_outcome", "not_proven");
    assert_eq!(v["commit_evidence"]["found"], false);
}

fn commit_fixture(surface: &str, transaction_id: i64, revision: i64) -> NewCommit {
    NewCommit {
        surface: surface.into(),
        transaction_id,
        revision,
        mode: "drive".into(),
        trigger_kind: "test".into(),
        trigger_ref: "fixture".into(),
        changed_inputs_json: r#"["input"]"#.into(),
        changed_derived_json: "[]".into(),
        changed_collections_json: "[]".into(),
        resource_commands_json: r#"[{"kind":"replace"}]"#.into(),
        output_frames_json: "[]".into(),
        command_count: 1,
        output_count: 0,
        effect_count: 1,
        suppressed_count: 0,
        noop: 0,
        oracle_status: Some("green".into()),
        oracle_error: None,
        duration_us: 10,
        graph_nodes: 3,
        graph_resources: 1,
        created_at: 100,
    }
}

fn record_commit(state: &std::sync::Arc<DaemonState>, commit: NewCommit) -> i64 {
    state.with_store(|s| s.record_commit(&commit)).unwrap()
}

fn record_receipt(
    state: &std::sync::Arc<DaemonState>,
    surface: &str,
    transaction_id: i64,
    revision: i64,
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
                created_at: 101,
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
