//! Cursor target validation.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn cursor_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("cursor:")
        .or_else(|| target.strip_prefix("cursor/"))
        .or_else(|| target.strip_prefix("cur:"))
        .or_else(|| target.strip_prefix("cur/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn cursor_evidence(state: &Arc<DaemonState>, target: &str, session_id: &str) -> Value {
    let graph_row = state
        .cursor
        .lock()
        .expect("cursor mutex poisoned")
        .state_rows()
        .into_iter()
        .find(|row| row.session == session_id);
    let session = match state.with_store(|s| s.get_session(session_id)) {
        Ok(row) => row,
        Err(e) => {
            return json!({
                "target": target,
                "session_id": session_id,
                "supported": true,
                "found": false,
                "graph_found": graph_row.is_some(),
                "error": e.to_string(),
                "summary": "cursor evidence could not read local session row",
                "reason": e.to_string(),
            });
        }
    };
    let graph_found = graph_row.is_some();
    let session_row_found = session.is_some();
    let session_alive = session.as_ref().is_some_and(|s| s.alive);
    let cursor_matches = session.as_ref().map(|s| {
        graph_row
            .as_ref()
            .is_some_and(|row| row.cursor == s.seen_cursor)
    });
    let ok = graph_found && cursor_matches != Some(false) && (!session_row_found || session_alive);

    json!({
        "target": target,
        "session_id": session_id,
        "supported": true,
        "found": graph_found || session_row_found,
        "graph_found": graph_found,
        "session_row_found": session_row_found,
        "session_alive": session_alive,
        "graph_cursor": graph_row.as_ref().map(|r| r.cursor).unwrap_or(0),
        "graph_last_frame": graph_row.as_ref().map(|r| r.last_frame.as_str()).unwrap_or(""),
        "graph_delta_since": graph_row.as_ref().and_then(|r| r.delta_since).unwrap_or(0),
        "local_seen_cursor": session.as_ref().map(|s| s.seen_cursor).unwrap_or(0),
        "cursor_matches_session": cursor_matches,
        "ok": ok,
        "summary": summary(session_id, graph_found, session_row_found, session_alive, ok),
        "reason": reason(graph_found, session_row_found, session_alive, ok),
    })
}

pub(super) fn push_cursor_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty() {
        "failed"
    } else if bool_at(evidence, "ok")
        || (bool_at(evidence, "graph_found") && !bool_at(evidence, "session_row_found"))
    {
        "passed"
    } else if !bool_at(evidence, "found") {
        "not_proven"
    } else {
        "failed"
    };
    checks.push(json!({
        "name": "cursor_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn summary(
    session_id: &str,
    graph_found: bool,
    session_row_found: bool,
    session_alive: bool,
    ok: bool,
) -> String {
    if ok {
        format!("cursor `{session_id}` projection agrees with local session row")
    } else if graph_found && !session_row_found {
        format!("cursor `{session_id}` has a graph projection but no local session row")
    } else if graph_found && !session_alive {
        format!("cursor `{session_id}` has a graph projection but local session is dead")
    } else if graph_found {
        format!("cursor `{session_id}` projection disagrees with local session row")
    } else if session_alive {
        format!("session `{session_id}` is alive locally but has no cursor projection")
    } else {
        format!("cursor `{session_id}` has no live graph or local session evidence")
    }
}

fn reason(
    graph_found: bool,
    session_row_found: bool,
    session_alive: bool,
    ok: bool,
) -> &'static str {
    if ok {
        ""
    } else if graph_found && !session_row_found {
        "cursor graph is live, but no local session row exists to verify the projection effect"
    } else if graph_found && !session_alive {
        "cursor graph is live, but the local session row is dead"
    } else if graph_found {
        "cursor graph projection does not match the local session row"
    } else if session_alive {
        "local session row is alive, but no cursor projection is live"
    } else {
        "no cursor projection or alive local session row was found"
    }
}
