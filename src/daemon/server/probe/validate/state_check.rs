//! Target-specific live-state evidence for `probe validate`.

use super::super::{state, DaemonState, SURFACES};
use serde_json::{json, Value};
use std::sync::Arc;

mod samples;

pub(super) fn all_surface_state_checks(
    daemon_state: &Arc<DaemonState>,
) -> (Vec<Value>, Vec<Value>) {
    let mut checks = Vec::new();
    let mut states = Vec::new();

    for surface in SURFACES {
        match state::state_value(
            daemon_state,
            &json!({ "verb": "state", "surface": surface }),
        ) {
            Ok(v) => {
                let (status, summary) = state_check_summary(&v, None, None);
                checks.push(json!({
                    "name": format!("state:{surface}"),
                    "status": status,
                    "summary": summary.clone(),
                }));
                states.push(annotated_surface_state(v, status, &summary));
            }
            Err(e) => {
                checks.push(json!({
                    "name": format!("state:{surface}"),
                    "status": "failed",
                    "summary": e.to_string(),
                }));
                states.push(json!({
                    "verb": "state",
                    "surface": surface,
                    "error": e.to_string(),
                    "rows": [],
                }));
            }
        }
    }

    (checks, states)
}

pub(super) fn annotated_surface_state(mut state: Value, status: &str, summary: &str) -> Value {
    let row_count = state
        .get("rows")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let samples = samples::sample_targets(&state, 3);
    if let Some(obj) = state.as_object_mut() {
        obj.insert("check_status".into(), json!(status));
        obj.insert("check_summary".into(), json!(summary));
        obj.insert("row_count".into(), json!(row_count));
        obj.insert("sample_targets".into(), json!(samples));
    }
    state
}

pub(super) fn state_check_summary(
    surface_state: &Value,
    handle: Option<&str>,
    why: Option<&Value>,
) -> (&'static str, String) {
    if let Some(resource) = expected_state_resource(handle, why) {
        let found = surface_state
            .get("rows")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .any(|row| row.get("resource_key").and_then(Value::as_str) == Some(resource.as_str()));
        if found {
            return ("passed", format!("target {resource} has a live state row"));
        }
        return (
            "failed",
            format!(
                "target {resource} has no live state row on surface {}",
                str_at(surface_state, "surface")
            ),
        );
    }

    let rows = surface_state.get("rows").and_then(Value::as_array);
    if str_at(surface_state, "surface") == "outbox" {
        return outbox_state_check_summary(rows);
    }

    let row_count = rows.map_or(0, Vec::len);
    if row_count == 0 {
        return (
            "not_proven",
            format!(
                "surface {} has no live state rows",
                str_at(surface_state, "surface")
            ),
        );
    }
    (
        "passed",
        format!(
            "surface {} has {row_count} live row(s)",
            str_at(surface_state, "surface")
        ),
    )
}

fn outbox_state_check_summary(rows: Option<&Vec<Value>>) -> (&'static str, String) {
    let empty = Vec::new();
    let rows = rows.unwrap_or(&empty);
    let row_count = rows.len();
    if row_count == 0 {
        return ("not_proven", "surface outbox has no live state rows".into());
    }

    let failed = rows
        .iter()
        .filter(|row| {
            !str_at(row, "last_error").is_empty() || failed_outbox_state(str_at(row, "state"))
        })
        .collect::<Vec<_>>();
    let pending = rows
        .iter()
        .filter(|row| pending_outbox_state(str_at(row, "state")))
        .collect::<Vec<_>>();

    if let Some(first) = failed.first() {
        return (
            "failed",
            format!(
                "surface outbox has {row_count} live row(s); {} failed publish row(s), first {}",
                failed.len(),
                outbox_handle(first)
            ),
        );
    }
    if let Some(first) = pending.first() {
        return (
            "not_proven",
            format!(
                "surface outbox has {row_count} live row(s); {} pending relay acceptance row(s), first {}",
                pending.len(),
                outbox_handle(first)
            ),
        );
    }

    (
        "passed",
        format!("surface outbox has {row_count} live published row(s)"),
    )
}

fn outbox_handle(row: &Value) -> String {
    let resource = str_at(row, "resource_key");
    if !resource.is_empty() {
        return resource.to_string();
    }
    row.get("local_id")
        .and_then(Value::as_i64)
        .map(|id| format!("outbox/{id}"))
        .unwrap_or_else(|| "outbox/<unknown>".to_string())
}

fn failed_outbox_state(state: &str) -> bool {
    matches!(state, "failed" | "error" | "rejected")
}

fn pending_outbox_state(state: &str) -> bool {
    matches!(state, "pending" | "queued" | "sending" | "")
}

pub(super) fn target_state_evidence(
    surface_state: &Value,
    handle: Option<&str>,
    why: Option<&Value>,
) -> Option<Value> {
    let resource = expected_state_resource(handle, why)?;
    let row = surface_state
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|row| row.get("resource_key").and_then(Value::as_str) == Some(resource.as_str()))
        .cloned();
    let found = row.is_some();
    let surface = str_at(surface_state, "surface");
    Some(json!({
        "kind": "state_row",
        "surface": surface,
        "handle": handle.unwrap_or(""),
        "resource_key": resource,
        "found": found,
        "row": row.unwrap_or(Value::Null),
        "summary": if found {
            format!("matched live state row on surface {surface}")
        } else {
            format!("no live state row on surface {surface}")
        },
        "reason": if found {
            ""
        } else {
            "the requested handle/resource is not materialized in the live surface graph"
        },
    }))
}

fn expected_state_resource(handle: Option<&str>, why: Option<&Value>) -> Option<String> {
    why.and_then(|v| v.get("resource_key").and_then(Value::as_str))
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| handle.and_then(resource_from_handle))
}

fn resource_from_handle(handle: &str) -> Option<String> {
    if let Some(channel) = handle.strip_prefix("sub:") {
        return (!channel.is_empty()).then(|| format!("sub/h/{channel}"));
    }
    if handle.starts_with("sub/") {
        return first_segments(handle, 3);
    }
    if let Some(id) = handle_id(handle, &["status:", "status/"]) {
        return Some(format!("status/{id}"));
    }
    if let Some(id) = handle_id(
        handle,
        &["turn:", "turn/", "turn_lifecycle:", "turn_lifecycle/"],
    ) {
        return Some(format!("turn_lifecycle/{id}"));
    }
    if let Some(id) = handle_id(handle, &["cursor:", "cursor/", "cur:", "cur/"]) {
        return Some(format!("cursor/{id}"));
    }
    if let Some(id) = handle_id(handle, &["outbox:", "outbox/"]) {
        return Some(format!("outbox/{id}"));
    }
    if let Some(id) = handle_id(handle, &["session_start:", "session_start/"]) {
        return Some(format!("session_start/{id}"));
    }
    if let Some(id) = handle_id(
        handle,
        &[
            "watch:",
            "watch/",
            "session_watch:",
            "session_watch/",
            "session-watch/",
        ],
    ) {
        return Some(format!("session-watch/{id}"));
    }
    if let Some(id) = handle_id(
        handle,
        &["hook:", "hook/", "hook_context:", "hook_context/"],
    ) {
        return Some(format!("hook/{id}/view"));
    }
    None
}

fn handle_id<'a>(handle: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    prefixes.iter().find_map(|prefix| {
        handle.strip_prefix(prefix).and_then(|rest| {
            let id = rest
                .split('/')
                .next()
                .unwrap_or(rest)
                .split('@')
                .next()
                .unwrap_or(rest);
            (!id.is_empty()).then_some(id)
        })
    })
}

fn first_segments(path: &str, count: usize) -> Option<String> {
    let parts = path.split('/').take(count).collect::<Vec<_>>();
    if parts.len() == count && parts.iter().all(|part| !part.is_empty()) {
        Some(parts.join("/"))
    } else {
        None
    }
}

fn str_at<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

#[cfg(test)]
mod tests;
