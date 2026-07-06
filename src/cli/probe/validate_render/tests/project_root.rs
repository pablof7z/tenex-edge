use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_project_root_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "project:room",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"project_root","status":"passed","summary":"project `room` root path `/tmp/room` exists"}
        ],
        "project_root_evidence": {
            "channel_h": "room",
            "project_root": "root",
            "channel_found": true,
            "found": true,
            "direct_binding_found": false,
            "inherited_binding": true,
            "binding_channel_h": "root",
            "abs_path": "/tmp/root",
            "path_absolute": true,
            "path_exists": true,
            "path_is_dir": true,
            "updated_at": 100
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("project root evidence"));
    assert!(text.contains("channel=room project_root=root"));
    assert!(text.contains("binding_channel=root path=/tmp/root"));
    assert!(text.contains("direct=false inherited=true"));
}

#[test]
fn validate_render_lists_project_root_reason() {
    let v = json!({
        "verb": "validate",
        "target": "project:missing",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"project_root","status":"not_proven","summary":"project `missing` has no local project root binding"}
        ],
        "limitations": ["no project_roots row exists for this channel or its top-level project root"],
        "project_root_evidence": {
            "channel_h": "missing",
            "project_root": "",
            "channel_found": false,
            "found": false,
            "direct_binding_found": false,
            "inherited_binding": false,
            "binding_channel_h": "",
            "reason": "no project_roots row exists for this channel or its top-level project root"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("project root evidence"));
    assert!(text.contains("no project_roots row exists"));
}
