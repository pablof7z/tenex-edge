use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_llm_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "llm:7",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"llm_outcome","status":"passed","summary":"llm call `7` exists and joins to 1 status receipt(s)"}
        ],
        "llm_evidence": {
            "llm_id": 7,
            "call_found": true,
            "session_id": "s1",
            "session_row_found": true,
            "session_alive": true,
            "provider": "ollama",
            "model": "glm",
            "window_hash": "sha256:w",
            "parsed_title": "T",
            "parsed_activity": "A",
            "system_prompt_bytes": 6,
            "transcript_slice_bytes": 16,
            "raw_response_bytes": 15,
            "receipt_count": 1,
            "receipt_artifacts": ["evt-status"],
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("llm evidence"));
    assert!(text.contains("llm:7 ollama / glm session=s1 (alive)"));
    assert!(text.contains("window=sha256:w receipts=1"));
    assert!(text.contains("bytes system=6 transcript=16 response=15"));
    assert!(text.contains("artifacts=evt-status"));
}
