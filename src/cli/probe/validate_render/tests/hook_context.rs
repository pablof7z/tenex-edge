use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_hook_context_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "hook:s1",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"hook_context_outcome","status":"passed","summary":"hook context `s1` live graph has a matching receipt"}
        ],
        "hook_context_evidence": {
            "session_id": "s1",
            "graph_found": true,
            "receipt_found": true,
            "revision_matches_receipt": true,
            "graph": {
                "revision": 4,
                "nodes": 7,
                "render_count": 2,
                "emitted": true,
                "text_bytes": 42,
                "rendered_unconfirmed_channel": false,
                "rendered_local_agents": true,
                "rendered_member_roster": true,
                "rendered_legacy_agents_roster": false,
                "local_agent_rows": 1,
                "member_rows": 2,
                "why_input_causes": ["hook/s1/presence", "hook/s1/messages"]
            },
            "member_roster_corroborated": true,
            "session_channel": {
                "channel_h": "room",
                "confirmed": true,
                "membership_snapshot": true,
                "member_count": 2,
                "admin_count": 1
            },
            "receipt": {
                "id": 9,
                "transaction_id": 3,
                "revision": 4,
                "artifact_ref": "s1:turn_start:100",
                "kind": "turn_start",
                "frame": "baseline",
                "shape": "full",
                "input_causes": ["presence", "messages"]
            },
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("hook context evidence"));
    assert!(text.contains("s1: graph=present receipt=present revision_match=true"));
    assert!(text.contains("graph revision=4 nodes=7 renders=2 emitted=true bytes=42"));
    assert!(text.contains("roster local_agents=true legacy_agents=false members=true corroborated=true local_rows=1 member_rows=2"));
    assert!(text.contains(
        "session channel=room confirmed=true membership_snapshot=true members=2 admins=1"
    ));
    assert!(text.contains("receipt id=9 txn=3 rev=4 kind=turn_start"));
    assert!(text.contains("receipt_causes=presence,messages"));
}

#[test]
fn validate_render_lists_hook_unconfirmed_channel_failure() {
    let v = json!({
        "verb": "validate",
        "target": "hook:s1",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"hook_context_outcome","status":"failed","summary":"hook context `s1` rendered an unconfirmed channel as active"}
        ],
        "hook_context_evidence": {
            "session_id": "s1",
            "graph_found": true,
            "receipt_found": true,
            "revision_matches_receipt": true,
            "graph": {
                "revision": 4,
                "nodes": 7,
                "render_count": 2,
                "emitted": true,
                "text_bytes": 88,
                "rendered_unconfirmed_channel": true,
                "missing_channel_warning_rendered": false,
                "rendered_local_agents": false,
                "rendered_member_roster": false,
                "rendered_legacy_agents_roster": false,
                "local_agent_rows": 0,
                "member_rows": 0,
                "why_input_causes": ["hook/s1/channel-meta"]
            },
            "member_roster_corroborated": true,
            "session_channel": {
                "channel_h": "ghost",
                "confirmed": false,
                "membership_snapshot": false,
                "member_count": 0,
                "admin_count": 0
            },
            "receipt": null,
            "reason": "hook context must render missing/unverified channels as degraded warnings, not normal channel blocks"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("rendered_unconfirmed_channel=true"));
    assert!(text.contains("session channel=ghost confirmed=false"));
    assert!(text.contains("degraded warnings"));
}

#[test]
fn validate_render_lists_hook_legacy_agent_roster_failure() {
    let v = json!({
        "verb": "validate",
        "target": "hook:s1",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"hook_context_outcome","status":"failed","summary":"hook context `s1` rendered local config as an agent roster"}
        ],
        "hook_context_evidence": {
            "session_id": "s1",
            "graph_found": true,
            "receipt_found": true,
            "revision_matches_receipt": true,
            "graph": {
                "revision": 4,
                "nodes": 7,
                "render_count": 2,
                "emitted": true,
                "text_bytes": 88,
                "rendered_local_agents": false,
                "rendered_member_roster": false,
                "rendered_legacy_agents_roster": true,
                "local_agent_rows": 1,
                "member_rows": 0,
                "why_input_causes": ["hook/s1/channel-meta"]
            },
            "member_roster_corroborated": false,
            "session_channel": {
                "channel_h": "room",
                "confirmed": true,
                "membership_snapshot": false,
                "member_count": 0,
                "admin_count": 0
            },
            "receipt": null,
            "reason": "configured local agents must render as local-agents; active channel roster must come from confirmed members/presence"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("roster local_agents=false legacy_agents=true members=false corroborated=false local_rows=1 member_rows=0"));
    assert!(text.contains("active channel roster must come from confirmed members/presence"));
}

#[test]
fn validate_render_lists_uncorroborated_member_roster_failure() {
    let v = json!({
        "verb": "validate",
        "target": "hook:s1",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"hook_context_outcome","status":"failed","summary":"hook context `s1` rendered an uncorroborated member roster"}
        ],
        "hook_context_evidence": {
            "session_id": "s1",
            "graph_found": true,
            "receipt_found": true,
            "revision_matches_receipt": true,
            "member_roster_corroborated": false,
            "graph": {
                "revision": 4,
                "nodes": 7,
                "render_count": 2,
                "emitted": true,
                "text_bytes": 88,
                "rendered_local_agents": false,
                "rendered_member_roster": true,
                "rendered_legacy_agents_roster": false,
                "local_agent_rows": 0,
                "member_rows": 2,
                "why_input_causes": ["hook/s1/members"]
            },
            "session_channel": {
                "channel_h": "room",
                "confirmed": true,
                "membership_snapshot": false,
                "member_count": 0,
                "admin_count": 0
            },
            "receipt": null,
            "reason": "hook context rendered members, but the session channel does not have a hydrated relay membership snapshot"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("members=true corroborated=false"));
    assert!(text.contains("membership_snapshot=false"));
    assert!(text.contains("hydrated relay membership snapshot"));
}
