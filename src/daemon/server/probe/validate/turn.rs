//! Turn lifecycle target validation.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn turn_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("turn:")
        .or_else(|| target.strip_prefix("turn/"))
        .or_else(|| target.strip_prefix("turn_lifecycle:"))
        .or_else(|| target.strip_prefix("turn_lifecycle/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn turn_evidence(state: &Arc<DaemonState>, target: &str, session_id: &str) -> Value {
    let graph_row = state
        .turn_lifecycle
        .lock()
        .expect("turn lifecycle mutex poisoned")
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
                "summary": "turn evidence could not read local session row",
                "reason": e.to_string(),
            });
        }
    };
    let graph_found = graph_row.is_some();
    let session_row_found = session.is_some();
    let session_alive = session.as_ref().is_some_and(|s| s.alive);
    let working_matches = session.as_ref().map(|s| {
        graph_row
            .as_ref()
            .is_some_and(|row| row.working == s.working)
    });
    let started_matches = session.as_ref().map(|s| {
        graph_row
            .as_ref()
            .is_some_and(|row| row.turn_started_at == s.turn_started_at)
    });
    let transcript_matches = session.as_ref().map(|s| {
        graph_row
            .as_ref()
            .is_some_and(|row| match &row.transcript_ref {
                Some(transcript) => s.transcript_path.as_deref() == Some(transcript.as_str()),
                None => true,
            })
    });
    let ok = graph_found
        && !matches!(working_matches, Some(false))
        && !matches!(started_matches, Some(false))
        && !matches!(transcript_matches, Some(false))
        && (!session_row_found || session_alive);

    json!({
        "target": target,
        "session_id": session_id,
        "supported": true,
        "found": graph_found || session_row_found,
        "graph_found": graph_found,
        "session_row_found": session_row_found,
        "session_alive": session_alive,
        "graph_working": graph_row.as_ref().map(|r| r.working).unwrap_or(false),
        "graph_turn_started_at": graph_row.as_ref().map(|r| r.turn_started_at).unwrap_or(0),
        "graph_transcript_ref": graph_row.as_ref().and_then(|r| r.transcript_ref.as_deref()).unwrap_or(""),
        "local_working": session.as_ref().map(|s| s.working).unwrap_or(false),
        "local_turn_started_at": session.as_ref().map(|s| s.turn_started_at).unwrap_or(0),
        "local_transcript_path": session.as_ref().and_then(|s| s.transcript_path.as_deref()).unwrap_or(""),
        "working_matches_session": working_matches,
        "started_matches_session": started_matches,
        "transcript_matches_session": transcript_matches,
        "ok": ok,
        "summary": summary(session_id, graph_found, session_row_found, session_alive, ok),
        "reason": reason(graph_found, session_row_found, session_alive, ok),
    })
}

pub(super) fn push_turn_check(
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
        "name": "turn_outcome",
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
        format!("turn `{session_id}` projection agrees with local session row")
    } else if graph_found && !session_row_found {
        format!("turn `{session_id}` has a graph projection but no local session row")
    } else if graph_found && !session_alive {
        format!("turn `{session_id}` has a graph projection but local session is dead")
    } else if graph_found {
        format!("turn `{session_id}` projection disagrees with local session row")
    } else if session_alive {
        format!("session `{session_id}` is alive locally but has no turn projection")
    } else {
        format!("turn `{session_id}` has no live graph or local session evidence")
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
        "turn graph is live, but no local session row exists to verify the projection effect"
    } else if graph_found && !session_alive {
        "turn graph is live, but the local session row is dead"
    } else if graph_found {
        "turn graph projection does not match the local session row"
    } else if session_alive {
        "local session row is alive, but no turn lifecycle projection is live"
    } else {
        "no turn lifecycle projection or alive local session row was found"
    }
}
