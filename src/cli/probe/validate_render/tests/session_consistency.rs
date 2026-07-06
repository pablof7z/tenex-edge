use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_session_consistency_evidence() {
    let v = json!({
        "verb": "validate",
        "target": null,
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"session_consistency","status":"failed","summary":"1/1 alive local session(s) have missing surface evidence"}
        ],
        "session_consistency": {
            "session_count": 1,
            "failed_count": 1,
            "live_projection_count": 1,
            "daemon_uptime_secs": 99,
            "reason": "one or more alive local sessions is missing status, session_watch, or active-channel subscription evidence",
            "rows": [{
                "session_id": "s1",
                "channel_h": "room",
                "ok": false,
                "missing": ["status", "session_watch", "sub/h"]
            }]
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("session consistency evidence"));
    assert!(text.contains("sessions=1 failed=1 live_projections=1 uptime=99s"));
    assert!(text.contains("s1 channel=room missing=status,session_watch,sub/h"));
}

#[test]
fn validate_render_lists_session_consistency_warmup() {
    let v = json!({
        "verb": "validate",
        "target": null,
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"session_consistency","status":"not_proven","summary":"1 alive local session(s) are waiting for live projections after daemon startup"}
        ],
        "session_consistency": {
            "session_count": 1,
            "failed_count": 1,
            "live_projection_count": 0,
            "daemon_uptime_secs": 2,
            "warmup_suspected": true,
            "reason": "daemon just started and no live session projections are populated yet; retry validation after status/watch/subscription warmup",
            "rows": [{
                "session_id": "s1",
                "channel_h": "room",
                "ok": false,
                "missing": ["status", "session_watch", "sub/h", "sub/d"]
            }]
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("sessions=1 failed=1 live_projections=0 uptime=2s"));
    assert!(text.contains("startup warmup suspected"));
    assert!(text.contains("retry validation after status/watch/subscription warmup"));
}
