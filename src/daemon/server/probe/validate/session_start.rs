//! Session-start outcome validation for advisory host effects.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn session_start_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("session_start:")
        .or_else(|| target.strip_prefix("session_start/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn session_start_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    session_id: &str,
) -> Value {
    let row = state
        .session_start
        .lock()
        .expect("session_start mutex poisoned")
        .state_rows()
        .into_iter()
        .find(|row| row.session_id == session_id);
    let Some(row) = row else {
        return json!({
            "target": target,
            "session_id": session_id,
            "supported": true,
            "found": false,
            "summary": format!("session_start `{session_id}` has no advisory row"),
            "reason": "no SessionStartRequested/SessionStarted/SessionStartFailed fact has reached the session_start graph for this session",
        });
    };

    let summary = match row.action.as_str() {
        "RecordStarted" => format!(
            "session_start `{}` recorded started in channel `{}`",
            row.session_id, row.channel_h
        ),
        "RecordFailed" => format!(
            "session_start `{}` failed at {}",
            row.session_id,
            row.failure_stage.as_deref().unwrap_or("unknown_stage")
        ),
        "Reassert" => format!(
            "session_start `{}` has a reassert intent but no recorded started outcome",
            row.session_id
        ),
        _ => format!(
            "session_start `{}` has a pending execute intent",
            row.session_id
        ),
    };
    let reason = match row.action.as_str() {
        "RecordStarted" => "",
        "RecordFailed" => row
            .failure_error
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("session_start failed but did not record an error message"),
        "Reassert" => {
            "reassert was planned, but no SessionStarted outcome fact has been recorded yet"
        }
        _ => "host effects are advisory until SessionStarted or SessionStartFailed is recorded",
    };

    json!({
        "target": target,
        "session_id": row.session_id,
        "supported": true,
        "found": true,
        "action": row.action,
        "channel_h": row.channel_h,
        "signer_pubkey": row.signer_pubkey,
        "reassert": row.reassert,
        "failure_stage": row.failure_stage,
        "failure_error": row.failure_error,
        "has_channel_ready_intent": row.has_channel_ready_intent,
        "has_spawn_intent": row.has_spawn_intent,
        "watch_pid": row.watch_pid,
        "ensure_subscription": row.ensure_subscription,
        "replay_chat": row.replay_chat,
        "summary": summary,
        "reason": reason,
    })
}

pub(super) fn push_session_start_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !bool_at(evidence, "found") {
        "not_proven"
    } else {
        match str_at(evidence, "action") {
            "RecordStarted" => "passed",
            "RecordFailed" => "failed",
            _ => "not_proven",
        }
    };
    checks.push(json!({
        "name": "session_start_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}
