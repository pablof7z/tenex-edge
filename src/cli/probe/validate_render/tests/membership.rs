use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_membership_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "admin:room:pk-admin",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"membership","status":"passed","summary":"pubkey `pk-admin` is admin in channel `room`"}
        ],
        "membership_evidence": {
            "target": "admin:room:pk-admin",
            "kind": "membership",
            "channel_h": "room",
            "pubkey": "pk-admin",
            "require_admin": true,
            "supported": true,
            "found": true,
            "ok": true,
            "channel_found": true,
            "membership_snapshot": true,
            "member_count": 3,
            "admin_count": 1,
            "role": "admin",
            "updated_at": 101,
            "profile_found": true,
            "profile_slug": "coder",
            "identity_found": true,
            "identity_session_id": "s1",
            "session_alive": true,
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("membership evidence"));
    assert!(text.contains("channel=room pubkey=pk-admin role=admin"));
    assert!(text.contains("members=3 admins=1 updated_at=101"));
    assert!(text.contains("profile=true slug=\"coder\" identity=true"));
}

#[test]
fn validate_render_lists_membership_snapshot_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "membership_snapshot:room",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"membership_snapshot","status":"passed","summary":"channel `room` has hydrated admin and member snapshots"}
        ],
        "membership_snapshot_evidence": {
            "target": "membership_snapshot:room",
            "kind": "membership_snapshot",
            "channel_h": "room",
            "supported": true,
            "found": true,
            "channel_found": true,
            "snapshot_complete": true,
            "admin_set_found": true,
            "member_set_found": true,
            "admin_set_updated_at": 101,
            "member_set_updated_at": 102,
            "set_count": 2,
            "member_count": 2,
            "admin_count": 1,
            "members": [
                {"pubkey":"pk-admin","role":"admin","updated_at":101},
                {"pubkey":"pk-member","role":"member","updated_at":102}
            ],
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("membership snapshot evidence"));
    assert!(text.contains("channel=room complete=true channel_found=true"));
    assert!(text.contains("admin_set=true updated_at=101 member_set=true updated_at=102"));
    assert!(text.contains("pk-admin role=admin updated_at=101"));
}
