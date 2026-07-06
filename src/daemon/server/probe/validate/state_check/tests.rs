use super::*;
use serde_json::json;

#[test]
fn surface_state_without_rows_is_not_proven() {
    let state = json!({ "surface": "status", "rows": [] });

    let (status, summary) = state_check_summary(&state, None, None);

    assert_eq!(status, "not_proven");
    assert!(summary.contains("no live state rows"));
}

#[test]
fn surface_state_with_rows_passes() {
    let state = json!({
        "surface": "status",
        "rows": [{ "resource_key": "status/s1" }]
    });

    let (status, summary) = state_check_summary(&state, None, None);

    assert_eq!(status, "passed");
    assert!(summary.contains("1 live row"));
}

#[test]
fn annotated_surface_state_adds_summary_and_sample_targets() {
    let state = json!({
        "surface": "status",
        "rows": [{
            "session": "s1",
            "resource_key": "status/s1"
        }]
    });

    let annotated = annotated_surface_state(state, "passed", "surface status has 1 live row");

    assert_eq!(annotated["check_status"], "passed");
    assert_eq!(annotated["row_count"], 1);
    assert_eq!(annotated["sample_targets"][0]["target"], "status:s1");
    assert_eq!(annotated["sample_targets"][0]["resource_key"], "status/s1");
}

#[test]
fn outbox_surface_with_publish_error_fails() {
    let state = json!({
        "surface": "outbox",
        "rows": [{
            "local_id": 13,
            "resource_key": "outbox/13",
            "state": "pending",
            "last_error": "relay rejected event",
        }]
    });

    let (status, summary) = state_check_summary(&state, None, None);

    assert_eq!(status, "failed");
    assert!(summary.contains("failed publish"));
    assert!(summary.contains("outbox/13"));
}

#[test]
fn outbox_surface_with_pending_rows_is_not_proven() {
    let state = json!({
        "surface": "outbox",
        "rows": [{
            "local_id": 14,
            "resource_key": "outbox/14",
            "state": "pending",
            "last_error": "",
        }]
    });

    let (status, summary) = state_check_summary(&state, None, None);

    assert_eq!(status, "not_proven");
    assert!(summary.contains("pending relay acceptance"));
}

#[test]
fn outbox_surface_with_published_rows_passes() {
    let state = json!({
        "surface": "outbox",
        "rows": [{
            "local_id": 15,
            "resource_key": "outbox/15",
            "state": "published",
            "last_error": "",
        }]
    });

    let (status, summary) = state_check_summary(&state, None, None);

    assert_eq!(status, "passed");
    assert!(summary.contains("live published"));
}
