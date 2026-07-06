use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_readiness_attempt_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "readiness_attempt:7",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"readiness_attempt","status":"passed","summary":"readiness attempt `7` verified channel `room`"}
        ],
        "readiness_attempt_evidence": {
            "id": 7,
            "found": true,
            "channel_h": "room",
            "expect_member": "pk-member",
            "source": "provider.ensure_channel_ready",
            "outcome": "ready",
            "attempt_reason": "channel readiness verified",
            "created_at": 100,
            "channel_found": true,
            "channel_name": "Room",
            "membership_snapshot": true,
            "member_count": 2,
            "admin_count": 1,
            "expected_member_found": true,
            "expected_member_role": "member",
            "current_ready": true,
            "reason": "channel readiness verified"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("readiness attempt evidence"));
    assert!(text.contains("attempt=7 channel=room outcome=ready"));
    assert!(text.contains("expected_member=pk-member found=true role=\"member\""));
    assert!(text.contains("channel readiness verified"));
}
