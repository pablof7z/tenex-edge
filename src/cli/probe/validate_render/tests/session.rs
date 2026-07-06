use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_session_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "session:s1",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"session_target","status":"failed","summary":"session `s1` is alive but missing live surface evidence"}
        ],
        "session_evidence": {
            "session_id": "s1",
            "found": true,
            "alive": true,
            "agent_slug": "coder",
            "harness": "codex",
            "channel_h": "room",
            "working": false,
            "last_seen": 100,
            "status_found": true,
            "watch_found": false,
            "sub_h_owned": true,
            "sub_d_owned": false,
            "missing": ["session_watch", "sub/d"],
            "reason": "alive session is missing status, session_watch, or active-channel subscription evidence"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("session evidence"));
    assert!(text.contains("s1: alive channel=room agent=coder harness=codex"));
    assert!(text.contains("status=yes watch=no sub_h=yes sub_d=no"));
    assert!(text.contains("missing=session_watch,sub/d"));
}
