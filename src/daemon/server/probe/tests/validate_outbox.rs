use super::*;

#[tokio::test]
async fn rpc_probe_validate_outbox_reports_failed_publish_error() {
    let state = DaemonState::new_for_test().await;
    let local_id = seed_durable_outbox(&state, "pending", Some("relay rejected event"), true);
    drive_outbox_enqueue(&state, local_id);
    drive_outbox_result(&state, local_id, false, Some("relay rejected event"));

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("outbox:{local_id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "outbox_outcome", "failed");
    assert_check_status(&v, "why", "passed");
    assert_check_status(&v, "state", "passed");
    assert_eq!(v["outbox_evidence"]["last_error"], "relay rejected event");
}

#[tokio::test]
async fn rpc_probe_validate_outbox_reports_published_row() {
    let state = DaemonState::new_for_test().await;
    let local_id = seed_durable_outbox(&state, "published", None, false);
    drive_outbox_enqueue(&state, local_id);
    drive_outbox_result(&state, local_id, true, None);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("outbox/{local_id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "outbox_outcome", "passed");
    assert_eq!(v["outbox_evidence"]["graph_state"], "published");
    assert_eq!(v["outbox_evidence"]["store_state"], "published");
}

#[tokio::test]
async fn rpc_probe_validate_outbox_accepts_historical_published_store_row() {
    let state = DaemonState::new_for_test().await;
    let local_id = seed_durable_outbox(&state, "published", None, false);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("outbox/{local_id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_eq!(v["verdict"], "passed_with_limitations");
    assert_check_status(&v, "outbox_outcome", "passed");
    assert_check_status(&v, "why", "not_proven");
    assert_check_status(&v, "state", "not_proven");
    assert_eq!(v["outbox_evidence"]["graph_found"], false);
    assert_eq!(v["outbox_evidence"]["store_state"], "published");
}

#[tokio::test]
async fn rpc_probe_validate_missing_outbox_reports_absent_evidence() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "outbox:404" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "outbox_outcome", "not_proven");
    assert_check_status(&v, "why", "failed");
    assert_check_status(&v, "state", "failed");
    assert_eq!(v["outbox_evidence"]["found"], false);
}

fn seed_durable_outbox(
    state: &std::sync::Arc<DaemonState>,
    state_name: &str,
    error: Option<&str>,
    bump_retries: bool,
) -> i64 {
    state.with_store(|s| {
        let id = s.enqueue_outbox(r#"{"id":"ev7"}"#, 100).unwrap();
        s.apply_outbox_projection(id, state_name, error, bump_retries)
            .unwrap();
        id
    })
}

fn drive_outbox_enqueue(state: &std::sync::Arc<DaemonState>, local_id: i64) {
    state
        .outbox
        .lock()
        .unwrap()
        .drive(InputFact::OutboxEnqueueApplied {
            local_id,
            event_id: "ev7".into(),
            event_hash: "sha256:event".into(),
            source_surface: "status".into(),
            source_ref: "status/s1#tx:1".into(),
            at: 100,
        })
        .unwrap();
}

fn drive_outbox_result(
    state: &std::sync::Arc<DaemonState>,
    local_id: i64,
    accepted: bool,
    error: Option<&str>,
) {
    state
        .outbox
        .lock()
        .unwrap()
        .drive(InputFact::RelayPublishAccepted {
            local_id,
            event_id: "ev7".into(),
            accepted,
            error: error.map(str::to_string),
            at: 120,
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
