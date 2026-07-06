//! Session target validation for hosted local sessions.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn session_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("session:")
        .and_then(|rest| rest.split('@').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn session_evidence(state: &Arc<DaemonState>, target: &str, session_id: &str) -> Value {
    let session = match state.with_store(|s| s.get_session(session_id)) {
        Ok(row) => row,
        Err(e) => {
            return json!({
                "target": target,
                "session_id": session_id,
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": "session evidence could not read local session row",
                "reason": e.to_string(),
            });
        }
    };
    let Some(session) = session else {
        return json!({
            "target": target,
            "session_id": session_id,
            "supported": true,
            "found": false,
            "summary": format!("session `{session_id}` has no local row"),
            "reason": "no local session row exists for this session id or alias",
        });
    };

    let status_found = state
        .status
        .lock()
        .expect("status mutex poisoned")
        .state_rows()
        .into_iter()
        .any(|row| row.session == session.session_id);
    let watch_found = state
        .session_watch
        .lock()
        .expect("session_watch mutex poisoned")
        .state_rows()
        .into_iter()
        .any(|row| row.session == session.session_id);
    let owner = format!("session-{}", session.session_id);
    let subs = state.subs.lock().expect("subs mutex poisoned").state_rows();
    let has_channel = !session.channel_h.trim().is_empty();
    let sub_h_owned = has_channel
        && owner_has_subscription(&subs, &format!("sub/h/{}", session.channel_h), &owner);
    let sub_d_owned = has_channel
        && owner_has_subscription(&subs, &format!("sub/d/{}", session.channel_h), &owner);
    let mut missing = Vec::new();
    if session.alive {
        if !status_found {
            missing.push("status");
        }
        if !watch_found {
            missing.push("session_watch");
        }
        if !has_channel {
            missing.push("active_channel");
        } else {
            if !sub_h_owned {
                missing.push("sub/h");
            }
            if !sub_d_owned {
                missing.push("sub/d");
            }
        }
    }
    let ok = session.alive && missing.is_empty();
    json!({
        "target": target,
        "session_id": session.session_id,
        "supported": true,
        "found": true,
        "alive": session.alive,
        "agent_slug": session.agent_slug,
        "harness": session.harness,
        "channel_h": session.channel_h,
        "working": session.working,
        "last_seen": session.last_seen,
        "status_found": status_found,
        "watch_found": watch_found,
        "sub_h_owned": sub_h_owned,
        "sub_d_owned": sub_d_owned,
        "missing": missing,
        "ok": ok,
        "summary": summary(session_id, session.alive, ok),
        "reason": reason(session.alive, ok),
    })
}

pub(super) fn push_session_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty() {
        "failed"
    } else if !bool_at(evidence, "found") {
        "not_proven"
    } else if !bool_at(evidence, "alive") {
        "not_proven"
    } else if bool_at(evidence, "ok") {
        "passed"
    } else {
        "failed"
    };
    checks.push(json!({
        "name": "session_target",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn owner_has_subscription(
    rows: &[crate::reconcile::subscriptions::probe::SubStateRow],
    resource_key: &str,
    owner: &str,
) -> bool {
    rows.iter().any(|row| {
        row.resource_key == resource_key && row.owners.iter().any(|candidate| candidate == owner)
    })
}

fn summary(session_id: &str, alive: bool, ok: bool) -> String {
    if ok {
        format!("session `{session_id}` agrees across status/watch/subscriptions")
    } else if alive {
        format!("session `{session_id}` is alive but missing live surface evidence")
    } else {
        format!("session `{session_id}` is not alive locally")
    }
}

fn reason(alive: bool, ok: bool) -> &'static str {
    if ok {
        ""
    } else if alive {
        "alive session is missing status, session_watch, or active-channel subscription evidence"
    } else {
        "dead or missing sessions can only be explained historically; live surface consistency is not proven"
    }
}
