use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_status_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "status:s1",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"status_outcome","status":"passed","summary":"status `s1` is published"}
        ],
        "status_evidence": {
            "session_id": "s1",
            "found": true,
            "graph_found": true,
            "session_row_found": true,
            "session_alive": true,
            "relay_status_found": true,
            "relay_status_live": true,
            "relay_status_count": 1,
            "relay_live_count": 1,
            "relay_channels": ["room"],
            "relay_live_channels": ["room"],
            "relay_pubkey": "pk-peer",
            "relay_slug": "coder",
            "relay_title": "T",
            "relay_activity": "reading",
            "relay_state": "working",
            "relay_expiration": 200,
            "graph_title": "T",
            "graph_state": "working",
            "graph_channels": ["room"],
            "channel_h": "room",
            "agent_slug": "coder",
            "harness": "codex",
            "local_working": true,
            "last_seen": 100,
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("status evidence"));
    assert!(text.contains("status/s1: graph=present session=alive"));
    assert!(text.contains("published state=working channels=room"));
    assert!(text.contains("local agent=coder harness=codex channel=room"));
    assert!(text.contains("relay status live=true rows=1 live_rows=1 channels=room"));
    assert!(text.contains("relay published pubkey=pk-peer slug=\"coder\""));
}
