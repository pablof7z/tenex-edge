use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_turn_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "turn:s1",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"turn_outcome","status":"passed","summary":"turn `s1` projection agrees with local session row"}
        ],
        "turn_evidence": {
            "session_id": "s1",
            "found": true,
            "graph_found": true,
            "session_row_found": true,
            "session_alive": true,
            "graph_working": true,
            "graph_turn_started_at": 100,
            "graph_transcript_ref": "tx1",
            "local_working": true,
            "local_turn_started_at": 100,
            "local_transcript_path": "tx1",
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("turn evidence"));
    assert!(text.contains("s1: graph=present session=alive"));
    assert!(text.contains("graph working=true started=100"));
    assert!(text.contains("local working=true started=100"));
}
