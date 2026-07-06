use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_quarantine_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "quarantine:evt-q",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"quarantine","status":"failed","summary":"event `evt-q` is quarantined before normal materialization"}
        ],
        "limitations": ["relay event is quarantined and has not been admitted to canonical event/message state"],
        "quarantine_evidence": {
            "event_prefix": "evt-q",
            "row_count": 1,
            "materialized": false,
            "message_found": false,
            "relay_event_found": false,
            "reason": "relay event is quarantined and has not been admitted to canonical event/message state",
            "rows": [{
                "id": "evt-q-123",
                "kind": 9,
                "pubkey": "pk-author",
                "channel_h": "room",
                "reason": "room roster is not hydrated",
                "quarantined_at": 120
            }]
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("quarantine evidence"));
    assert!(text.contains("event_prefix=evt-q"));
    assert!(text.contains("evt-q-123 kind=9 channel=room"));
    assert!(text.contains("room roster is not hydrated"));
}

#[test]
fn validate_render_event_lists_quarantine_reason() {
    let v = json!({
        "verb": "validate",
        "target": "event:evt-q",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"event","status":"failed","summary":"event `evt-q-123` is quarantined before normal materialization"}
        ],
        "event_evidence": {
            "found": true,
            "requested_id": "evt-q",
            "event_id": "evt-q-123",
            "quarantine_found": true,
            "quarantine_count": 1,
            "quarantine_rows": [{"reason":"author is not an admitted member"}],
            "reason": "relay event is quarantined and has not been admitted to canonical event/message state"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("quarantine rows=1 reason=author is not an admitted member"));
}
