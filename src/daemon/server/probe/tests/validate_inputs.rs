use super::*;

mod targets;

#[tokio::test]
async fn rpc_probe_validate_explains_unowned_facts_without_erroring() {
    let state = DaemonState::new_for_test().await;
    let cases = vec![
        (
            InputFact::RelayEventObserved {
                event_id: "ev1".into(),
                kind: 1,
                channel_h: Some("room".into()),
                pubkey: "pk".into(),
                at: 100,
            },
            "RelayEventObserved",
            "event_ingest",
        ),
        (InputFact::ClockTick { at: 102 }, "ClockTick", "timekeeping"),
    ];

    for (fact, kind, frontier) in cases {
        let v = rpc_probe(&state, &json!({ "verb": "validate", "fact": fact })).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["verdict"], "passed_with_limitations");
        assert!(v["surface"].is_null());
        assert!(v["simulate"].is_null());
        assert_check_status(&v, "fact", "not_proven");
        assert_eq!(v["fact_evidence"]["kind"], kind);
        assert_eq!(v["fact_evidence"]["frontier"], frontier);
        assert_eq!(v["fact_evidence"]["supported"], false);
        assert!(v["fact_evidence"]["reason"]
            .as_str()
            .unwrap()
            .contains("no"));
    }
}

#[tokio::test]
async fn rpc_probe_validate_explains_unknown_targets_without_erroring() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(&state, &json!({ "verb": "validate", "target": "wat" })).unwrap();

    assert_eq!(v["ok"], true);
    assert_eq!(v["verdict"], "passed_with_limitations");
    assert_eq!(v["target_evidence"]["supported"], false);
    assert_eq!(v["target_evidence"]["kind"], "unknown_target");
    assert_check_status(&v, "target", "not_proven");
    assert!(v["target_evidence"]["reason"]
        .as_str()
        .unwrap()
        .contains("surface"));
}

#[tokio::test]
async fn rpc_probe_validate_reports_malformed_parameters_inside_envelope() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(
        &state,
        &json!({
            "verb": "validate",
            "target": ["status:s1"],
            "capsule": { "id": 1 },
            "cause": false,
            "since": "yesterday"
        }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_eq!(v["verdict"], "failed");
    assert_check_status(&v, "input", "failed");
    let params = v["parameter_evidence"].as_array().unwrap();
    assert_eq!(params.len(), 4);
    assert!(params.iter().any(|p| p["parameter"] == "target"));
    assert!(params.iter().any(|p| p["parameter"] == "capsule"));
    assert!(params.iter().any(|p| p["parameter"] == "cause"));
    assert!(params.iter().any(|p| p["parameter"] == "since"));
    assert!(v["target_evidence"].is_null());
    assert!(v["replay"].is_null());
}

#[tokio::test]
async fn rpc_probe_validate_reports_empty_parameters_inside_envelope() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(
        &state,
        &json!({
            "verb": "validate",
            "target": "",
            "capsule": "",
            "cause": ""
        }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_eq!(v["verdict"], "failed");
    assert_check_status(&v, "input", "failed");
    let params = v["parameter_evidence"].as_array().unwrap();
    assert_eq!(params.len(), 3);
    assert!(params
        .iter()
        .all(|p| p["reason"].as_str().unwrap().contains("non-empty")));
    assert!(v["replay"].is_null());
}

#[tokio::test]
async fn rpc_probe_validate_reports_invalid_fact_shape_inside_envelope() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "fact": { "Bogus": {} } }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_eq!(v["verdict"], "failed");
    assert!(v["surface"].is_null());
    assert!(v["simulate"].is_null());
    assert_check_status(&v, "fact", "failed");
    assert_eq!(v["fact_evidence"]["kind"], "InvalidInputFact");
    assert_eq!(v["fact_evidence"]["valid"], false);
    assert_eq!(v["fact_evidence"]["frontier"], "input_decode");
    assert!(v["fact_evidence"]["reason"]
        .as_str()
        .unwrap()
        .contains("Bogus"));

    let text = rpc_probe(&state, &json!({ "verb": "validate", "fact": "not json" })).unwrap();

    assert_eq!(text["ok"], false);
    assert_eq!(text["verdict"], "failed");
    assert_check_status(&text, "fact", "failed");
    assert_eq!(text["fact_evidence"]["kind"], "InvalidInputFact");
    assert_eq!(text["fact_evidence"]["frontier"], "input_decode");
    assert!(text["fact_evidence"]["reason"]
        .as_str()
        .unwrap()
        .contains("invalid fact JSON"));
}

#[tokio::test]
async fn rpc_probe_validate_reports_invalid_capsule_inside_envelope() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "capsule:not-an-id" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_eq!(v["verdict"], "failed");
    assert!(v["capsule"].is_null());
    assert!(v["replay"].is_null());
    assert!(v["replay_error"].is_null());
    assert_check_status(&v, "target", "failed");
    assert_no_check(&v, "replay");
    assert_eq!(v["target_evidence"]["kind"], "invalid_capsule");
    assert_eq!(v["target_evidence"]["valid"], false);
    assert!(v["target_evidence"]["reason"]
        .as_str()
        .unwrap()
        .contains("capsule id must be an integer"));

    let param = rpc_probe(
        &state,
        &json!({ "verb": "validate", "capsule": "not-an-id" }),
    )
    .unwrap();

    assert_eq!(param["ok"], false);
    assert_eq!(param["verdict"], "failed");
    assert!(param["capsule"].is_null());
    assert!(param["replay"].is_null());
    assert!(param["replay_error"].is_null());
    assert_check_status(&param, "input", "failed");
    assert_no_check(&param, "replay");
    assert!(param["parameter_evidence"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["parameter"] == "capsule"
            && p["reason"]
                .as_str()
                .unwrap()
                .contains("integer replay capsule id")));
}

#[tokio::test]
async fn rpc_probe_validate_reports_empty_capsule_target_inside_envelope() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(&state, &json!({ "verb": "validate", "target": "capsule:" })).unwrap();

    assert_eq!(v["ok"], false);
    assert_eq!(v["verdict"], "failed");
    assert!(v["capsule"].is_null());
    assert!(v["replay"].is_null());
    assert!(v["replay_error"].is_null());
    assert_check_status(&v, "target", "failed");
    assert_eq!(v["target_evidence"]["kind"], "empty_handle");
    assert_eq!(v["target_evidence"]["valid"], false);
    assert!(v["target_evidence"]["reason"]
        .as_str()
        .unwrap()
        .contains("replay capsule id"));
}

fn assert_check_status(v: &serde_json::Value, name: &str, status: &str) {
    let row = check_row(v, name);
    assert_eq!(row["status"], status);
}

fn check_row<'a>(v: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    v["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == name)
        .expect("check row")
}

fn assert_no_check(v: &serde_json::Value, name: &str) {
    assert!(!v["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["name"] == name));
}
