use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_cursor_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "cursor:s1",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"cursor_outcome","status":"passed","summary":"cursor `s1` projection agrees with local session row"}
        ],
        "cursor_evidence": {
            "session_id": "s1",
            "found": true,
            "graph_found": true,
            "session_row_found": true,
            "session_alive": true,
            "graph_cursor": 25,
            "graph_last_frame": "HookFrame",
            "graph_delta_since": 10,
            "local_seen_cursor": 25,
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("cursor evidence"));
    assert!(text.contains("s1: graph=present session=alive"));
    assert!(text.contains("graph cursor=25 frame=HookFrame"));
    assert!(text.contains("local seen_cursor=25"));
}
