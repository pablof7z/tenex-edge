use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_txn_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "txn:status:7",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"txn_outcome","status":"passed","summary":"txn `status:7` has durable commit evidence and 1 receipt(s)"}
        ],
        "txn_evidence": {
            "surface": "status",
            "transaction_id": 7,
            "commit_count": 1,
            "receipt_count": 1,
            "total_commit_count": 1,
            "total_receipt_count": 1,
            "receipt_revisions_match_commits": true,
            "ambiguous": false,
            "latest_commit": {
                "id": 11,
                "revision": 3,
                "created_at": 100,
                "mode": "drive",
                "trigger_kind": "test",
                "trigger_ref": "fixture",
                "effect_count": 1,
                "command_count": 1,
                "output_count": 0,
                "noop": false
            },
            "receipts": [{
                "id": 12,
                "revision": 3,
                "artifact_ref": "evt-status-7"
            }],
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("transaction evidence"));
    assert!(text.contains("status:7 commits=1 receipts=1"));
    assert!(text.contains("commit id=11 rev=3 at=100 mode=drive"));
    assert!(text.contains("receipt id=12 rev=3 artifact=evt-status-7"));
}
