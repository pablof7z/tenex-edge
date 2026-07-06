use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_state_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "sub:room",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"state","status":"passed","summary":"target sub/h/room has a live state row"}
        ],
        "state_evidence": {
            "surface": "subscriptions",
            "resource_key": "sub/h/room",
            "found": true,
            "row": {
                "resource_key": "sub/h/room",
                "refcount": 2,
                "owners": ["daemon", "s1"]
            },
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("state evidence"));
    assert!(text.contains("sub/h/room: found on subscriptions"));
    assert!(text.contains("refcount=2 owners=2"));
}

#[test]
fn validate_render_lists_aggregate_outbox_state_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "all",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {
                "name":"state:outbox",
                "status":"failed",
                "summary":"surface outbox has 1 live row(s); 1 failed publish row(s), first outbox/13"
            }
        ],
        "surface_states": [{
            "surface": "outbox",
            "rows": [{
                "local_id": 13,
                "resource_key": "outbox/13",
                "state": "pending",
                "retries": 68,
                "event_id": "e9db050d0587e0b4bafe13d4e5713ae43ffd65a8321ad4eb84366f16d1d0b7f2",
                "source_ref": "",
                "last_error": "relay rejected event: blocked: unknown member"
            }]
        }]
    });

    let text = render_validate(&v);

    assert!(text.contains("aggregate state evidence"));
    assert!(text.contains("outbox/13 state=pending retries=68 event=e9db050d0587..."));
    assert!(text.contains("relay rejected event: blocked: unknown member"));
}

#[test]
fn validate_render_lists_aggregate_surface_summaries() {
    let v = json!({
        "verb": "validate",
        "target": "all",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {
                "name":"state:status",
                "status":"passed",
                "summary":"surface status has 1 live row(s)"
            }
        ],
        "surface_states": [{
            "surface": "status",
            "check_status": "passed",
            "check_summary": "surface status has 1 live row(s)",
            "row_count": 1,
            "sample_targets": [{
                "target": "status:s1",
                "resource_key": "status/s1"
            }],
            "rows": [{
                "session": "s1",
                "resource_key": "status/s1"
            }]
        }]
    });

    let text = render_validate(&v);

    assert!(text.contains("aggregate state evidence"));
    assert!(text.contains("status passed rows=1 sample=status:s1"));
}

#[test]
fn validate_render_lists_direct_surface_state_summary() {
    let v = json!({
        "verb": "validate",
        "target": "state:status",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {
                "name":"state",
                "status":"passed",
                "summary":"surface status has 1 live row(s)"
            }
        ],
        "state": {
            "surface": "status",
            "check_status": "passed",
            "check_summary": "surface status has 1 live row(s)",
            "row_count": 1,
            "sample_targets": [{
                "target": "status:s1",
                "resource_key": "status/s1"
            }],
            "rows": [{
                "session": "s1",
                "resource_key": "status/s1"
            }]
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("aggregate state evidence"));
    assert!(text.contains("status passed rows=1 sample=status:s1"));
}
