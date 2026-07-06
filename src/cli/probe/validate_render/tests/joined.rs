use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_joined_channel_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "joined:s1:side",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"joined_channels","status":"passed","summary":"session `s1` is joined to `side` with subscription coverage"}
        ],
        "joined_evidence": {
            "session_id": "s1",
            "active_channel_h": "room",
            "session_alive": true,
            "joined_count": 2,
            "channel_h": "side",
            "requested_joined": true,
            "missing_subscription_count": 0,
            "rows": [
                {
                    "channel_h": "room",
                    "joined_at": 100,
                    "channel_found": true,
                    "sub_h_owned": true,
                    "sub_d_owned": true
                },
                {
                    "channel_h": "side",
                    "joined_at": 101,
                    "channel_found": true,
                    "sub_h_owned": true,
                    "sub_d_owned": true
                }
            ]
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("joined-channel evidence"));
    assert!(text.contains("session=s1 active=room alive=true joined=2 requested=\"side\""));
    assert!(text.contains("side joined_at=101 channel_found=true sub_h=true sub_d=true"));
}

#[test]
fn validate_render_lists_joined_channel_reason() {
    let v = json!({
        "verb": "validate",
        "target": "joined:s1:side",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"joined_channels","status":"failed","summary":"session `s1` has 1 joined channel(s) without subscription coverage"}
        ],
        "joined_evidence": {
            "session_id": "s1",
            "active_channel_h": "room",
            "session_alive": true,
            "joined_count": 1,
            "channel_h": "side",
            "requested_joined": true,
            "missing_subscription_count": 1,
            "rows": [{
                "channel_h": "side",
                "joined_at": 100,
                "channel_found": true,
                "sub_h_owned": true,
                "sub_d_owned": false
            }],
            "reason": "one or more joined channels is missing sub/h or sub/d subscription coverage"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("missing_subscription_count=1"));
    assert!(text.contains("missing sub/h or sub/d subscription coverage"));
}
