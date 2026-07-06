use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_unknown_target_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "not-a-known-target",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"target","status":"not_proven","summary":"target `not-a-known-target` is not a known validation target"}
        ],
        "limitations": ["target must be a surface, probe handle, explain handle, or `capsule:<id>`"],
        "target_evidence": {
            "target": "not-a-known-target",
            "supported": false,
            "kind": "unknown_target",
            "reason": "target must be a surface, probe handle, explain handle, or `capsule:<id>`"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("target evidence"));
    assert!(text.contains("not-a-known-target: not proven"));
    assert!(text.contains("target must be a surface"));
}

#[test]
fn validate_render_marks_invalid_target_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "status/",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"target","status":"failed","summary":"target `status/` is missing a status resource"}
        ],
        "target_evidence": {
            "target": "status/",
            "supported": false,
            "valid": false,
            "kind": "empty_handle",
            "reason": "target `status/` must include a non-empty status resource after `status/`"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("target evidence"));
    assert!(text.contains("status/: invalid"));
    assert!(text.contains("non-empty status resource"));
}

#[test]
fn validate_render_avoids_double_colon_for_colon_suffixed_targets() {
    let v = json!({
        "verb": "validate",
        "target": "capsule:",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"target","status":"failed","summary":"target `capsule:` is missing a replay capsule id"}
        ],
        "target_evidence": {
            "target": "capsule:",
            "supported": false,
            "valid": false,
            "kind": "empty_handle",
            "reason": "target `capsule:` must include a non-empty replay capsule id after `capsule:`"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("capsule: invalid"));
    assert!(!text.contains("capsule:: invalid"));
}
