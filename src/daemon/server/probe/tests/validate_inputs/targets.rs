use super::*;

#[tokio::test]
async fn rpc_probe_validate_accepts_planner_label_without_space() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "planner:status/s1/activity" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert!(v["target_evidence"].is_null());
    assert_check_status(&v, "cause_label", "passed");
    assert_eq!(v["surface"], "status");
    assert_eq!(v["cause_label_evidence"]["label"], "status/s1/activity");
    assert_eq!(v["cause_label_evidence"]["kind"], "planner_label");
}

#[tokio::test]
async fn rpc_probe_validate_reports_malformed_planner_labels_inside_envelope() {
    let state = DaemonState::new_for_test().await;
    let cases = [
        ("planner:", "planner label"),
        ("planner: ", "missing a label"),
        ("planner:status/", "empty path segments"),
        ("planner:outbox:not-an-id", "not probe handle shorthand"),
    ];

    for (target, reason) in cases {
        let v = rpc_probe(&state, &json!({ "verb": "validate", "target": target })).unwrap();

        assert_eq!(v["ok"], false);
        assert_eq!(v["verdict"], "failed");
        assert!(v["cause_label_evidence"].is_null());
        assert_check_status(&v, "target", "failed");
        assert!(matches!(
            v["target_evidence"]["kind"].as_str().unwrap(),
            "empty_handle" | "invalid_planner_label"
        ));
        assert_eq!(v["target_evidence"]["valid"], false);
        assert!(v["target_evidence"]["reason"]
            .as_str()
            .unwrap()
            .contains(reason));
    }
}

#[tokio::test]
async fn rpc_probe_validate_reports_malformed_visible_resource_paths_inside_envelope() {
    let state = DaemonState::new_for_test().await;
    let cases = [
        ("sub/h/", "empty path segments"),
        ("sub/room", "`sub/<h|d|p>/<id>`"),
        ("sub/x/room", "`sub/<h|d|p>/<id>`"),
        ("outbox/not-an-id/event_id", "integer local id"),
    ];

    for (target, reason) in cases {
        let v = rpc_probe(&state, &json!({ "verb": "validate", "target": target })).unwrap();

        assert_eq!(v["ok"], false);
        assert_eq!(v["verdict"], "failed");
        assert!(v["why"].is_null());
        assert!(v["state"].is_null());
        assert_check_status(&v, "target", "failed");
        assert_no_check(&v, "state");
        assert_eq!(v["target_evidence"]["kind"], "invalid_resource_path");
        assert_eq!(v["target_evidence"]["valid"], false);
        assert!(v["target_evidence"]["reason"]
            .as_str()
            .unwrap()
            .contains(reason));
    }
}

#[tokio::test]
async fn rpc_probe_validate_reports_malformed_known_handles_inside_envelope() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "outbox:not-an-id" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_eq!(v["verdict"], "failed");
    assert_eq!(v["surface"], "outbox");
    assert!(v["why"].is_null());
    assert!(v["state"].is_null());
    assert_check_status(&v, "target", "failed");
    assert_no_check(&v, "why");
    assert_no_check(&v, "state");
    assert_eq!(v["target_evidence"]["kind"], "invalid_probe_handle");
    assert_eq!(v["target_evidence"]["valid"], false);
    assert!(v["target_evidence"]["reason"]
        .as_str()
        .unwrap()
        .contains("integer local id"));
}
