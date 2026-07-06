//! Visible Trellis resource-path validation for `probe validate`.

use serde_json::{json, Value};

pub(super) fn malformed_resource_path_evidence(target: Option<&str>) -> Option<Value> {
    let target = target?;
    if has_empty_segment(target) && is_visible_resource_path(target) {
        return Some(invalid_resource_path(
            target,
            "visible Trellis resource paths must not contain empty path segments",
        ));
    }
    if let Some(rest) = target.strip_prefix("sub/") {
        return malformed_subscription_path(target, rest);
    }
    if let Some(rest) = target.strip_prefix("outbox/") {
        let id = rest.split('/').next().unwrap_or(rest);
        if id.parse::<i64>().is_err() {
            return Some(invalid_resource_path(
                target,
                "outbox visible resource paths must use an integer local id",
            ));
        }
    }
    None
}

fn malformed_subscription_path(target: &str, rest: &str) -> Option<Value> {
    let (space, entity) = rest.split_once('/').unwrap_or((rest, ""));
    if !matches!(space, "h" | "d" | "p") {
        return Some(invalid_resource_path(
            target,
            "subscription visible resource paths must be `sub/<h|d|p>/<id>`",
        ));
    }
    if entity.is_empty() {
        return Some(invalid_resource_path(
            target,
            "subscription visible resource paths must include a non-empty id",
        ));
    }
    None
}

fn is_visible_resource_path(target: &str) -> bool {
    const PREFIXES: [&str; 13] = [
        "sub/",
        "status/",
        "turn/",
        "turn_lifecycle/",
        "cursor/",
        "cur/",
        "outbox/",
        "session_start/",
        "watch/",
        "session_watch/",
        "session-watch/",
        "hook/",
        "hook_context/",
    ];
    PREFIXES.iter().any(|prefix| target.starts_with(prefix))
}

fn has_empty_segment(target: &str) -> bool {
    target.split('/').any(str::is_empty)
}

fn invalid_resource_path(target: &str, reason: &str) -> Value {
    json!({
        "target": target,
        "supported": false,
        "valid": false,
        "kind": "invalid_resource_path",
        "summary": format!("target `{target}` is not a valid visible Trellis resource path"),
        "reason": reason,
    })
}
