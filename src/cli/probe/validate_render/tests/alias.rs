use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_alias_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "tmux_pane:%1",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"alias","status":"passed","summary":"alias `tmux_pane:%1` resolves to live session `s1` with surface evidence"}
        ],
        "alias_evidence": {
            "alias_kind": "tmux_pane",
            "harness": null,
            "external_id": "%1",
            "row_count": 1,
            "resolved_live": true,
            "resolved_session_id": "s1",
            "session_found": true,
            "session_alive": true,
            "channel_h": "room",
            "agent_slug": "codex",
            "status_found": true,
            "watch_found": true,
            "sub_h_owned": true,
            "sub_d_owned": true,
            "missing": [],
            "rows": [{
                "harness": "codex",
                "external_id_kind": "tmux_pane",
                "external_id": "%1",
                "session_id": "s1",
                "session_alive": true,
                "created_at": 100
            }]
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("alias evidence"));
    assert!(text.contains("kind=tmux_pane"));
    assert!(text.contains("session alive=true channel=room agent=codex"));
    assert!(text.contains("codex:tmux_pane:%1 -> s1"));
}

#[test]
fn validate_render_lists_alias_missing_surfaces() {
    let v = json!({
        "verb": "validate",
        "target": "alias:codex:harness_session:native-s1",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"alias","status":"failed","summary":"alias resolves with missing surface evidence"}
        ],
        "alias_evidence": {
            "alias_kind": "harness_session",
            "harness": "codex",
            "external_id": "native-s1",
            "row_count": 1,
            "resolved_live": true,
            "resolved_session_id": "s1",
            "session_found": true,
            "session_alive": true,
            "channel_h": "room",
            "agent_slug": "codex",
            "status_found": false,
            "watch_found": true,
            "sub_h_owned": false,
            "sub_d_owned": true,
            "missing": ["status", "sub/h"],
            "rows": [],
            "reason": "alive session resolved by alias is missing status, session_watch, or active-channel subscription evidence"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("status=no watch=yes sub_h=no sub_d=yes"));
    assert!(text.contains("missing=status,sub/h"));
}
