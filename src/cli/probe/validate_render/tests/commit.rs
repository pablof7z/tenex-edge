use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_commit_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "commit:11",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"commit_outcome","status":"passed","summary":"commit `11` is a valid `status` txn 7 rev 3"}
        ],
        "commit_evidence": {
            "commit_id": 11,
            "found": true,
            "surface": "status",
            "transaction_id": 7,
            "revision": 3,
            "mode": "drive",
            "trigger_kind": "test",
            "trigger_ref": "fixture",
            "command_count": 1,
            "command_json_count": 1,
            "output_count": 0,
            "output_json_count": 0,
            "effect_count": 1,
            "suppressed_count": 0,
            "noop": false,
            "payload_valid": true,
            "candidate_receipt_count": 1,
            "matching_receipt_count": 1,
            "receipt_delta_ms": 1,
            "oracle_status": "green",
            "graph_nodes": 3,
            "graph_resources": 1,
            "created_at": 100,
            "receipts": [{
                "id": 12,
                "revision": 3,
                "artifact_ref": "evt-status-7"
            }],
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("commit evidence"));
    assert!(text.contains("id=11 status:7 rev=3 mode=drive"));
    assert!(text.contains("commands=1/1 outputs=0/0 effects=1"));
    assert!(text.contains("payload_valid=true receipts=1/1 delta_ms=1 oracle=green"));
    assert!(text.contains("receipt id=12 rev=3 artifact=evt-status-7"));
}

#[test]
fn validate_render_lists_missing_commit_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "commit:404",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"commit_outcome","status":"not_proven","summary":"commit `404` was not found"}
        ],
        "commit_evidence": {
            "commit_id": 404,
            "found": false,
            "reason": "no trellis_commits row exists for this id"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("commit evidence"));
    assert!(text.contains("commit/404: not found"));
    assert!(text.contains("no trellis_commits row exists for this id"));
}
