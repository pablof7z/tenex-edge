use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_receipt_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "receipt:12",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"receipt_outcome","status":"passed","summary":"receipt `12` matches a durable commit and has valid payload JSON"}
        ],
        "receipt_evidence": {
            "receipt_id": 12,
            "surface": "status",
            "transaction_id": 7,
            "revision": 3,
            "created_at": 101,
            "artifact_ref": "evt-status-7",
            "changed_summary_valid": true,
            "commands_valid": true,
            "command_count": 1,
            "artifact_receipt_count": 1,
            "commit_count": 1,
            "matching_commit_count": 1,
            "revision_matches_commit": true,
            "commit_delta_ms": 1,
            "nearest_commit": {
                "id": 11,
                "revision": 3,
                "created_at": 100,
                "mode": "drive",
                "trigger_kind": "test",
                "trigger_ref": "fixture",
                "effect_count": 1,
                "command_count": 1,
                "noop": false
            },
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("receipt evidence"));
    assert!(text.contains("id=12 surface=status txn=7 rev=3"));
    assert!(text.contains("commit_match=true matches=1 total_commits=1"));
    assert!(text.contains("nearest commit id=11 rev=3 at=100 mode=drive"));
}
