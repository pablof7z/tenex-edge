use super::*;
use crate::state::receipts::NewReceipt;
use crate::state::trellis_commits::NewCommit;

#[tokio::test]
async fn rpc_probe_validate_receipt_matches_commit_revision() {
    let state = DaemonState::new_for_test().await;
    record_commit(&state, "status", 7, 3);
    let id = record_receipt(&state, "status", 7, 3, r#"[{"kind":"publish"}]"#);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("receipt:{id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "receipt_outcome", "passed");
    assert_eq!(v["receipt_evidence"]["surface"], "status");
    assert_eq!(v["receipt_evidence"]["matching_commit_count"], 1);
    assert_eq!(v["receipt_evidence"]["revision_matches_commit"], true);
    assert_eq!(v["receipt_evidence"]["command_count"], 1);
    assert_eq!(v["receipt_evidence"]["artifact_receipt_count"], 1);
}

#[tokio::test]
async fn rpc_probe_validate_receipt_fails_without_matching_commit() {
    let state = DaemonState::new_for_test().await;
    let id = record_receipt(&state, "status", 9, 5, "[]");

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("receipt:{id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "receipt_outcome", "failed");
    assert_eq!(v["receipt_evidence"]["commit_count"], 0);
    assert_eq!(v["receipt_evidence"]["revision_matches_commit"], false);
}

#[tokio::test]
async fn rpc_probe_validate_receipt_fails_invalid_payload_json() {
    let state = DaemonState::new_for_test().await;
    record_commit(&state, "status", 7, 3);
    let id = record_receipt(&state, "status", 7, 3, "not-json");

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("receipt:{id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "receipt_outcome", "failed");
    assert_eq!(v["receipt_evidence"]["commands_valid"], false);
}

fn record_commit(
    state: &std::sync::Arc<DaemonState>,
    surface: &str,
    transaction_id: i64,
    revision: i64,
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
    commands: &str,
) -> i64 {
    state
        .with_store(|s| {
            s.record_receipt(&NewReceipt {
                surface: surface.into(),
                transaction_id,
                revision,
                changed_summary: r#"{"changed":true}"#.into(),
                commands: commands.into(),
                artifact_ref: Some(format!("evt-{surface}-{transaction_id}")),
                created_at: 101,
            })
        })
        .unwrap()
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
