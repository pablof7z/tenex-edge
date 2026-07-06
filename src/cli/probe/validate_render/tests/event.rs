use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_event_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "event:evt-status",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"event","status":"passed","summary":"event `evt-status-long` has 1 Trellis receipt(s)"}
        ],
        "event_evidence": {
            "requested_id": "evt-status",
            "event_id": "evt-status-long",
            "found": true,
            "receipt_count": 1,
            "receipt_surfaces": ["status"],
            "message_found": true,
            "message_channel_h": "room",
            "message_sync_state": "accepted",
            "native_event_id": "evt-status-long",
            "outbox_store_count": 1,
            "outbox_graph_count": 1,
            "outbox_found": true,
            "outbox_published": true,
            "outbox_pending": false,
            "outbox_failed": false,
            "outbox_rows": [{
                "local_id": 7,
                "state": "published",
                "retries": 0,
                "event_json_id": "evt-status-long"
            }],
            "relay_event_found": true,
            "relay_kind": 9,
            "relay_channel_h": "room",
            "relay_author_pubkey": "pk-author",
            "relay_content_len": 15,
            "relay_tags_valid": true,
            "relay_tag_count": 2,
            "relay_channel_found": true,
            "relay_channel_name": "Room",
            "relay_author_profile_found": true,
            "relay_author_slug": "author",
            "relay_membership_snapshot": true,
            "relay_author_role": "member",
            "reason": "Trellis receipts explain this event artifact"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("event evidence"));
    assert!(text.contains("evt-status-long: materialized"));
    assert!(text.contains("receipts=1 surfaces=status"));
    assert!(text.contains("message sync=accepted channel=room"));
    assert!(text.contains("outbox store=1 graph=1 published=true"));
    assert!(text.contains("outbox row id=7 state=published"));
    assert!(text
        .contains("relay kind=9 channel=room author=pk-author content_len=15 tags=2 valid=true"));
    assert!(text.contains("relay channel_found=true name=\"Room\" profile=true slug=\"author\""));
}
