//! Outbox target evidence for `probe validate`.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn outbox_target(target: &str) -> Option<i64> {
    target
        .strip_prefix("outbox:")
        .or_else(|| target.strip_prefix("outbox/"))
        .and_then(|rest| rest.split('/').next())
        .and_then(|id| id.parse::<i64>().ok())
}

pub(super) fn outbox_evidence(state: &Arc<DaemonState>, target: &str, local_id: i64) -> Value {
    let graph_row = state
        .outbox
        .lock()
        .expect("outbox mutex poisoned")
        .state_rows()
        .into_iter()
        .find(|row| row.local_id == local_id);
    let store_row = match state.with_store(|s| s.get_outbox(local_id)) {
        Ok(row) => row,
        Err(e) => {
            return json!({
                "target": target,
                "local_id": local_id,
                "kind": "outbox",
                "supported": true,
                "found": graph_row.is_some(),
                "graph_found": graph_row.is_some(),
                "store_row_found": false,
                "summary": format!("outbox/{local_id} evidence failed: {e}"),
                "reason": e.to_string(),
                "error": e.to_string(),
            });
        }
    };
    let graph_state = graph_row
        .as_ref()
        .map(|row| row.state.as_str())
        .unwrap_or("");
    let store_state = store_row
        .as_ref()
        .map(|row| row.state.as_str())
        .unwrap_or("");
    let last_error = graph_row
        .as_ref()
        .and_then(|row| row.last_error.as_deref())
        .or_else(|| store_row.as_ref().and_then(|row| row.last_error.as_deref()))
        .unwrap_or("");
    let found = graph_row.is_some() || store_row.is_some();
    let mismatched = state_mismatch(graph_state, store_state);

    json!({
        "target": target,
        "local_id": local_id,
        "kind": "outbox",
        "supported": true,
        "found": found,
        "graph_found": graph_row.is_some(),
        "store_row_found": store_row.is_some(),
        "graph_state": graph_state,
        "store_state": store_state,
        "state_mismatch": mismatched,
        "event_id": graph_row.as_ref().map(|row| row.event_id.as_str()).unwrap_or(""),
        "source_ref": graph_row.as_ref().map(|row| row.source_ref.as_str()).unwrap_or(""),
        "graph_retries": graph_row.as_ref().map(|row| row.retries).unwrap_or(0),
        "store_retries": store_row.as_ref().map(|row| row.retries).unwrap_or(0),
        "last_error": last_error,
        "enqueued_at": store_row.as_ref().map(|row| row.enqueued_at).unwrap_or(0),
        "event_json_id": store_row.as_ref().and_then(|row| event_json_id(&row.event_json)).unwrap_or_default(),
        "summary": summary(local_id, graph_state, store_state, last_error, found, mismatched),
        "reason": reason(graph_state, store_state, last_error, found, mismatched),
    })
}

pub(super) fn push_outbox_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty()
        || bool_at(evidence, "state_mismatch")
        || failed_state(str_at(evidence, "graph_state"))
        || failed_state(str_at(evidence, "store_state"))
        || !str_at(evidence, "last_error").is_empty()
    {
        "failed"
    } else if !bool_at(evidence, "found")
        || pending_state(first_state(
            str_at(evidence, "graph_state"),
            str_at(evidence, "store_state"),
        ))
    {
        "not_proven"
    } else {
        "passed"
    };
    checks.push(json!({
        "name": "outbox_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn summary(
    local_id: i64,
    graph_state: &str,
    store_state: &str,
    last_error: &str,
    found: bool,
    mismatched: bool,
) -> String {
    if !found {
        return format!("outbox/{local_id} has no graph or durable queue row");
    }
    if mismatched {
        return format!("outbox/{local_id} graph/store state mismatch");
    }
    if !last_error.is_empty() {
        return format!("outbox/{local_id} publish failed: {last_error}");
    }
    let state = first_state(graph_state, store_state);
    match state {
        "published" => format!("outbox/{local_id} is published"),
        "pending" => format!("outbox/{local_id} is still pending relay acceptance"),
        other if !other.is_empty() => format!("outbox/{local_id} is {other}"),
        _ => format!("outbox/{local_id} has incomplete outbox state"),
    }
}

fn reason(
    graph_state: &str,
    store_state: &str,
    last_error: &str,
    found: bool,
    mismatched: bool,
) -> &'static str {
    if !found {
        return "no Trellis outbox row or durable outbox queue row exists for this local id";
    }
    if mismatched {
        return "Trellis outbox projection and durable queue row disagree on publish state";
    }
    if !last_error.is_empty() || failed_state(graph_state) || failed_state(store_state) {
        return "durable outbox row records a failed relay publish outcome";
    }
    if pending_state(first_state(graph_state, store_state)) {
        return "outbox publish is still pending; relay acceptance has not been proven";
    }
    ""
}

fn state_mismatch(graph_state: &str, store_state: &str) -> bool {
    !graph_state.is_empty() && !store_state.is_empty() && graph_state != store_state
}

fn first_state<'a>(graph_state: &'a str, store_state: &'a str) -> &'a str {
    if !graph_state.is_empty() {
        graph_state
    } else {
        store_state
    }
}

fn failed_state(state: &str) -> bool {
    matches!(state, "failed" | "error" | "rejected")
}

fn pending_state(state: &str) -> bool {
    matches!(state, "pending" | "queued" | "sending" | "")
}

fn event_json_id(event_json: &str) -> Option<String> {
    serde_json::from_str::<Value>(event_json)
        .ok()
        .and_then(|v| v.get("id").and_then(Value::as_str).map(str::to_string))
}
