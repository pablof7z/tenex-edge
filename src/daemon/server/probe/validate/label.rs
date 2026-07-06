//! Cause-label evidence for `probe validate`.

use super::target::surface_from_probe_handle;
use serde_json::{json, Value};

pub(super) fn cause_label_evidence(target: Option<&str>) -> Option<Value> {
    let target = target?;
    let label = planner_label_target(target).unwrap_or(target);
    if let Some(v) = subscriptions_label_evidence(target, label) {
        return Some(v);
    }
    if let Some(v) = session_watch_label_evidence(target, label) {
        return Some(v);
    }
    planner_label_target(target).and_then(|label| {
        planner_label_surface(label).map(|surface| {
            json!({
                "target": target,
                "label": label,
                "surface": surface,
                "kind": "planner_label",
                "supported": true,
                "summary": format!("planner label `{label}` belongs to {surface}"),
                "reason": "planner labels name Trellis nodes/collections, not standalone live resources",
            })
        })
    })
}

pub(super) fn malformed_planner_label_evidence(target: Option<&str>) -> Option<Value> {
    let target = target?;
    let label = planner_label_target(target)?;
    if label.is_empty() {
        return Some(invalid_planner_label(
            target,
            label,
            "planner target is missing a label after `planner:`",
        ));
    }
    if label.contains(':') && surface_from_probe_handle(label).is_some() {
        return Some(invalid_planner_label(
            target,
            label,
            "planner labels must use visible Trellis label paths, not probe handle shorthand",
        ));
    }
    if label.split('/').any(str::is_empty) && surface_from_probe_handle(label).is_some() {
        return Some(invalid_planner_label(
            target,
            label,
            "planner labels must not contain empty path segments",
        ));
    }
    None
}

fn planner_label_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("planner: ")
        .or_else(|| target.strip_prefix("planner:"))
        .or_else(|| target.strip_prefix("planner/"))
}

fn subscriptions_label_evidence(target: &str, label: &str) -> Option<Value> {
    let rest = label.strip_prefix("subscriptions/")?;
    let (scope, name) = rest.split_once('/')?;
    let known = match scope {
        "daemon" => matches!(
            name,
            "channels" | "addressed_pubkeys" | "archived_channels" | "live_channels" | "subs"
        ),
        "session" => {
            let (_, field) = name.split_once('/')?;
            matches!(field, "channels" | "subs")
        }
        _ => false,
    };
    known.then(|| {
        json!({
            "target": target,
            "label": label,
            "surface": "subscriptions",
            "kind": "cause_label",
            "supported": true,
            "summary": format!("cause label `{label}` belongs to subscriptions"),
            "reason": "subscription cause labels identify Trellis inputs or planner collections, not individual relay resources",
        })
    })
}

fn session_watch_label_evidence(target: &str, label: &str) -> Option<Value> {
    let rest = label.strip_prefix("session_watch/")?;
    matches!(rest, "live_sessions" | "watched_sessions" | "resources").then(|| {
        json!({
            "target": target,
            "label": label,
            "surface": "session_watch",
            "kind": "cause_label",
            "supported": true,
            "summary": format!("cause label `{label}` belongs to session_watch"),
            "reason": "session_watch cause labels identify Trellis graph inputs or planner collections, not one watched-session resource",
        })
    })
}

fn planner_label_surface(label: &str) -> Option<&'static str> {
    if label.contains(':') || label.split('/').any(str::is_empty) {
        return None;
    }
    surface_from_probe_handle(label)
}

fn invalid_planner_label(target: &str, label: &str, reason: &str) -> Value {
    json!({
        "target": target,
        "label": label,
        "supported": false,
        "valid": false,
        "kind": "invalid_planner_label",
        "summary": format!("planner label `{label}` is malformed"),
        "reason": reason,
    })
}
