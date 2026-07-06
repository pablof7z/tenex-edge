use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_outbox_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "outbox:13",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"outbox_outcome","status":"failed","summary":"outbox/13 publish failed: relay rejected"}
        ],
        "outbox_evidence": {
            "local_id": 13,
            "found": true,
            "graph_found": true,
            "store_row_found": true,
            "graph_state": "pending",
            "store_state": "pending",
            "event_id": "ev7",
            "event_json_id": "ev7",
            "source_ref": "status/s1#tx:1",
            "graph_retries": 1,
            "store_retries": 1,
            "enqueued_at": 100,
            "last_error": "relay rejected",
            "reason": "durable outbox row records a failed relay publish outcome"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("outbox evidence"));
    assert!(text.contains("outbox/13: graph=pending store=pending"));
    assert!(text.contains("event_id=ev7 durable_event_id=ev7"));
    assert!(text.contains("source=status/s1#tx:1"));
    assert!(text.contains("error: relay rejected"));
}
