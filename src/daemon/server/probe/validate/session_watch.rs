//! Session-watch validation for advisory process-liveness tracking.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn session_watch_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("watch:")
        .or_else(|| target.strip_prefix("watch/"))
        .or_else(|| target.strip_prefix("session_watch:"))
        .or_else(|| target.strip_prefix("session_watch/"))
        .or_else(|| target.strip_prefix("session-watch/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn session_watch_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    session_id: &str,
) -> Value {
    let graph_row = state
        .session_watch
        .lock()
        .expect("session_watch mutex poisoned")
        .state_rows()
        .into_iter()
        .find(|row| row.session == session_id);
    let session = match state.with_store(|s| s.get_session(session_id)) {
        Ok(session) => session,
        Err(e) => {
            return json!({
                "target": target,
                "session_id": session_id,
                "supported": true,
                "found": false,
                "graph_open": graph_row.is_some(),
                "error": e.to_string(),
                "summary": "session_watch evidence could not read local session row",
                "reason": e.to_string(),
            });
        }
    };
    let child_pid = session.as_ref().and_then(|s| s.child_pid);
    let process_alive = child_pid.map(super::super::super::engine_lifecycle::pid_alive);
    let graph_open = graph_row.is_some();
    let session_alive = session.as_ref().is_some_and(|s| s.alive);
    let summary = summary(
        session_id,
        graph_open,
        session_alive,
        child_pid,
        process_alive,
    );
    let reason = reason(graph_open, session.as_ref(), child_pid, process_alive);

    json!({
        "target": target,
        "session_id": session_id,
        "supported": true,
        "found": graph_open || session.is_some(),
        "graph_open": graph_open,
        "resource_key": graph_row.as_ref().map(|r| r.resource_key.as_str()).unwrap_or(""),
        "refcount": graph_row.as_ref().map(|r| r.refcount).unwrap_or(0),
        "owners": graph_row.as_ref().map(|r| r.owners.clone()).unwrap_or_default(),
        "session_row_found": session.is_some(),
        "session_alive": session_alive,
        "channel_h": session.as_ref().map(|s| s.channel_h.as_str()).unwrap_or(""),
        "agent_slug": session.as_ref().map(|s| s.agent_slug.as_str()).unwrap_or(""),
        "child_pid": child_pid,
        "process_alive": process_alive,
        "last_seen": session.as_ref().map(|s| s.last_seen).unwrap_or(0),
        "summary": summary,
        "reason": reason,
    })
}

pub(super) fn push_session_watch_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty() {
        "failed"
    } else if bool_at(evidence, "graph_open")
        && bool_at(evidence, "session_alive")
        && evidence.get("process_alive").and_then(Value::as_bool) == Some(true)
    {
        "passed"
    } else if (bool_at(evidence, "graph_open")
        && bool_at(evidence, "session_alive")
        && evidence.get("child_pid").is_some_and(Value::is_null))
        || (!bool_at(evidence, "graph_open") && !bool_at(evidence, "session_row_found"))
    {
        "not_proven"
    } else {
        "failed"
    };
    checks.push(json!({
        "name": "session_watch_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn summary(
    session_id: &str,
    graph_open: bool,
    session_alive: bool,
    child_pid: Option<i32>,
    process_alive: Option<bool>,
) -> String {
    match (graph_open, session_alive, child_pid, process_alive) {
        (true, true, Some(pid), Some(true)) => {
            format!("session_watch `{session_id}` is open and pid {pid} is alive")
        }
        (true, true, Some(pid), Some(false)) => {
            format!("session_watch `{session_id}` is open but pid {pid} is not alive")
        }
        (true, true, None, _) => {
            format!("session_watch `{session_id}` is open but has no recorded child_pid")
        }
        (true, false, _, _) => {
            format!("session_watch `{session_id}` is open without an alive local session row")
        }
        (false, true, _, _) => {
            format!("session `{session_id}` is alive locally but has no open session_watch row")
        }
        _ => format!("session_watch `{session_id}` has no live graph or local session evidence"),
    }
}

fn reason(
    graph_open: bool,
    session: Option<&crate::state::Session>,
    child_pid: Option<i32>,
    process_alive: Option<bool>,
) -> &'static str {
    match (graph_open, session, child_pid, process_alive) {
        (true, Some(session), Some(_), Some(true)) if session.alive => "",
        (true, Some(session), Some(_), Some(false)) if session.alive => {
            "recorded child_pid is not alive; session_watch has not observed or applied ProcessExited"
        }
        (true, Some(session), None, _) if session.alive => {
            "no child_pid is recorded, so process liveness cannot be checked"
        }
        (true, Some(_), _, _) => "session_watch graph is open but local session row is dead",
        (true, None, _, _) => "session_watch graph is open but local session row is missing",
        (false, Some(session), _, _) if session.alive => {
            "local session row is alive but session_watch graph has no open watch"
        }
        _ => "no SessionStarted fact has opened this watch and no alive local session row exists",
    }
}
