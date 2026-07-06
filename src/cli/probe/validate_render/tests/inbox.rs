use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_inbox_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "inbox:evt-in",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"inbox","status":"passed","summary":"inbox `evt-in` has 1 completed inbound row(s)"}
        ],
        "inbox_evidence": {
            "event_prefix": "evt-in",
            "target_session": null,
            "row_count": 1,
            "event_count": 1,
            "pending_count": 0,
            "processing_count": 0,
            "delivered_count": 1,
            "failed_count": 0,
            "rows": [{
                "event_id": "evt-in",
                "target_session": "s1",
                "target_kind": "session",
                "state": "delivered",
                "channel_h": "room",
                "session_alive": true,
                "body_len": 11
            }],
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("inbox evidence"));
    assert!(text.contains("event_prefix=evt-in"));
    assert!(text.contains("evt-in -> s1 (session) state=delivered"));
}
