use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_subscription_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "sub:room",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"subscription_outcome","status":"passed","summary":"subscription `room` has all 2 expected channel resource(s)"}
        ],
        "subscription_evidence": {
            "kind": "channel",
            "entity": "room",
            "revision": 4,
            "expected_resource_count": 2,
            "found_resource_count": 2,
            "receipt_count": 1,
            "resources": [{
                "resource_key": "sub/h/room",
                "found": true,
                "refcount": 2,
                "owners": ["daemon-subs", "session-s1"],
                "input_causes": ["subscriptions/daemon/channels"]
            }, {
                "resource_key": "sub/d/room",
                "found": true,
                "refcount": 2,
                "owners": ["daemon-subs", "session-s1"],
                "input_causes": ["subscriptions/daemon/channels"]
            }],
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("subscription evidence"));
    assert!(text.contains("room (channel) resources=2/2 receipts=1 revision=4"));
    assert!(text.contains("sub/h/room: live refcount=2 owners=daemon-subs,session-s1"));
    assert!(text.contains("causes=subscriptions/daemon/channels"));
}
