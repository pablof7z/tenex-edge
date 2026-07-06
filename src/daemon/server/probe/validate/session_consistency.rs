//! Cross-surface consistency checks for local alive sessions.

use super::report::{bool_at, int_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

const WARMUP_WINDOW_SECS: u64 = 15;

pub(super) fn session_consistency_evidence(state: &Arc<DaemonState>) -> Value {
    let sessions = match state.with_store(|s| s.list_alive_sessions()) {
        Ok(rows) => rows,
        Err(e) => {
            return json!({
                "kind": "session_consistency",
                "supported": true,
                "error": e.to_string(),
                "session_count": 0,
                "failed_count": 0,
                "rows": [],
                "summary": "could not read local alive sessions",
                "reason": e.to_string(),
            });
        }
    };
    let status_sessions = state
        .status
        .lock()
        .expect("status mutex poisoned")
        .state_rows()
        .into_iter()
        .map(|row| row.session)
        .collect::<BTreeSet<_>>();
    let watched_sessions = state
        .session_watch
        .lock()
        .expect("session_watch mutex poisoned")
        .state_rows()
        .into_iter()
        .map(|row| row.session)
        .collect::<BTreeSet<_>>();
    let subscriptions = state.subs.lock().expect("subs mutex poisoned").state_rows();

    let rows = sessions
        .iter()
        .map(|session| {
            let owner = format!("session-{}", session.session_id);
            let has_channel = !session.channel_h.trim().is_empty();
            let sub_h = has_channel
                && owner_has_subscription(
                    &subscriptions,
                    &format!("sub/h/{}", session.channel_h),
                    &owner,
                );
            let sub_d = has_channel
                && owner_has_subscription(
                    &subscriptions,
                    &format!("sub/d/{}", session.channel_h),
                    &owner,
                );
            let status = status_sessions.contains(&session.session_id);
            let watch = watched_sessions.contains(&session.session_id);
            let mut missing = Vec::new();
            if !status {
                missing.push("status");
            }
            if !watch {
                missing.push("session_watch");
            }
            if !has_channel {
                missing.push("active_channel");
            } else {
                if !sub_h {
                    missing.push("sub/h");
                }
                if !sub_d {
                    missing.push("sub/d");
                }
            }
            json!({
                "session_id": session.session_id,
                "agent_slug": session.agent_slug,
                "channel_h": session.channel_h,
                "status_found": status,
                "watch_found": watch,
                "sub_h_owned": sub_h,
                "sub_d_owned": sub_d,
                "missing": missing,
                "ok": missing.is_empty(),
            })
        })
        .collect::<Vec<_>>();
    let failed_count = rows.iter().filter(|row| !bool_at(row, "ok")).count();
    let session_count = rows.len();
    let daemon_uptime_secs = crate::util::now_secs().saturating_sub(state.started_at);
    let live_projection_count =
        status_sessions.len() + watched_sessions.len() + subscriptions.len();
    let warmup_suspected = session_count > 0
        && failed_count == session_count
        && live_projection_count == 0
        && daemon_uptime_secs <= WARMUP_WINDOW_SECS;
    json!({
        "kind": "session_consistency",
        "supported": true,
        "session_count": session_count,
        "failed_count": failed_count,
        "daemon_started_at": state.started_at,
        "daemon_uptime_secs": daemon_uptime_secs,
        "live_projection_count": live_projection_count,
        "warmup_suspected": warmup_suspected,
        "rows": rows,
        "summary": summary(session_count, failed_count, warmup_suspected),
        "reason": reason(session_count, failed_count, warmup_suspected),
    })
}

pub(super) fn push_session_consistency_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty() {
        "failed"
    } else if int_at(evidence, "session_count") == 0 {
        "not_proven"
    } else if bool_at(evidence, "warmup_suspected") {
        "not_proven"
    } else if int_at(evidence, "failed_count") > 0 {
        "failed"
    } else {
        "passed"
    };
    checks.push(json!({
        "name": "session_consistency",
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

fn summary(session_count: usize, failed_count: usize, warmup_suspected: bool) -> String {
    if session_count == 0 {
        "no alive local sessions to cross-check".into()
    } else if warmup_suspected {
        format!(
            "{session_count} alive local session(s) are waiting for live projections after daemon startup"
        )
    } else if failed_count == 0 {
        format!("{session_count} alive local session(s) agree across status/watch/subscriptions")
    } else {
        format!(
            "{failed_count}/{session_count} alive local session(s) have missing surface evidence"
        )
    }
}

fn reason(session_count: usize, failed_count: usize, warmup_suspected: bool) -> &'static str {
    if session_count == 0 {
        "no local alive session rows exist, so cross-surface session consistency cannot be proven"
    } else if warmup_suspected {
        "daemon just started and no live session projections are populated yet; retry validation after status/watch/subscription warmup"
    } else if failed_count > 0 {
        "one or more alive local sessions is missing status, session_watch, or active-channel subscription evidence"
    } else {
        ""
    }
}
