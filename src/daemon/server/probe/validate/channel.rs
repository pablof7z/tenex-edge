//! Channel target evidence for `probe validate`.

use super::super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

const CHANNEL_LIMITATION: &str = "channel provisioning is a host/provider side effect; no provider readiness attempt is recorded for this channel";
const READINESS_LIMITATION: &str =
    "channel readiness is advisory unless relay metadata and membership snapshots are hydrated";

pub(super) fn channel_evidence(state: &Arc<DaemonState>, target: &str, channel_h: &str) -> Value {
    let mode = if is_readiness_target(target) {
        "readiness"
    } else {
        "channel"
    };
    let readiness = readiness_evidence(state, channel_h);
    match state.with_store(|store| {
        let Some(channel) = store.get_channel(channel_h)? else {
            return Ok::<Value, anyhow::Error>(json!({
                "target": target,
                "channel_h": channel_h,
                "kind": mode,
                "supported": true,
                "found": false,
                "summary": format!("channel `{channel_h}` is not materialized from relay kind:39000"),
                "reason": channel_reason(false, false, &readiness),
                "readiness_ok": false,
                "readiness_summary": readiness_summary(false, false, &readiness),
                "readiness_reason": readiness_reason(false, false, &readiness),
                "session_start_count": readiness.rows.len(),
                "session_start_channel_ready_count": readiness.channel_ready_count,
                "session_start_failed_count": readiness.failed_count,
                "channel_ready_failure_count": readiness.channel_ready_failure_count,
                "provider_attempt_count": readiness.provider_attempt_count,
                "provider_degraded_count": readiness.provider_degraded_count,
                "provider_attempt_rows": readiness.provider_rows,
                "session_start_rows": readiness.rows,
            }));
        };

        let members = store.list_channel_members(channel_h)?;
        let admin_count = members.iter().filter(|m| m.role == "admin").count();
        let member_count = members.len();
        let membership_snapshot = store.has_channel_membership_snapshot(channel_h)?;
        let project_root = store.channel_project_root(channel_h)?;
        let is_root = project_root.as_deref() == Some(channel_h);
        let human_name = channel.human_name().map(str::to_string);

        Ok::<Value, anyhow::Error>(json!({
            "target": target,
            "channel_h": channel_h,
            "kind": mode,
            "supported": true,
            "found": true,
            "summary": format!("channel `{channel_h}` is materialized from relay kind:39000"),
            "reason": channel_reason(true, membership_snapshot, &readiness),
            "name": channel.name,
            "human_name": human_name,
            "about": channel.about,
            "parent": channel.parent,
            "project_root": project_root,
            "is_root": is_root,
            "is_archived": channel.is_archived(),
            "membership_snapshot": membership_snapshot,
            "admin_count": admin_count,
            "member_count": member_count,
            "created_at": channel.created_at,
            "updated_at": channel.updated_at,
            "readiness_ok": membership_snapshot,
            "readiness_summary": readiness_summary(true, membership_snapshot, &readiness),
            "readiness_reason": readiness_reason(true, membership_snapshot, &readiness),
            "session_start_count": readiness.rows.len(),
            "session_start_channel_ready_count": readiness.channel_ready_count,
            "session_start_failed_count": readiness.failed_count,
            "channel_ready_failure_count": readiness.channel_ready_failure_count,
            "provider_attempt_count": readiness.provider_attempt_count,
            "provider_degraded_count": readiness.provider_degraded_count,
            "provider_attempt_rows": readiness.provider_rows,
            "session_start_rows": readiness.rows,
        }))
    }) {
        Ok(v) => v,
        Err(e) => json!({
            "target": target,
            "channel_h": channel_h,
            "kind": mode,
            "supported": true,
            "found": false,
            "summary": format!("channel `{channel_h}` evidence failed: {e}"),
            "reason": channel_reason(false, false, &readiness),
            "error": e.to_string(),
            "readiness_ok": false,
            "readiness_summary": readiness_summary(false, false, &readiness),
            "readiness_reason": e.to_string(),
            "session_start_count": readiness.rows.len(),
            "session_start_channel_ready_count": readiness.channel_ready_count,
            "session_start_failed_count": readiness.failed_count,
            "channel_ready_failure_count": readiness.channel_ready_failure_count,
            "provider_attempt_count": readiness.provider_attempt_count,
            "provider_degraded_count": readiness.provider_degraded_count,
            "provider_attempt_rows": readiness.provider_rows,
            "session_start_rows": readiness.rows,
        }),
    }
}

pub(super) fn push_channel_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let readiness = str_at(evidence, "kind") == "readiness";
    let status = if readiness {
        if !str_at(evidence, "error").is_empty() {
            "failed"
        } else if bool_at(evidence, "readiness_ok") {
            "passed"
        } else if int_at(evidence, "channel_ready_failure_count") > 0
            || int_at(evidence, "provider_degraded_count") > 0
        {
            "failed"
        } else {
            "not_proven"
        }
    } else if bool_at(evidence, "found") {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": if readiness { "channel_readiness" } else { "channel" },
        "status": status,
        "summary": if readiness {
            str_at(evidence, "readiness_summary")
        } else {
            str_at(evidence, "summary")
        },
    }));
    let reason = if readiness {
        str_at(evidence, "readiness_reason")
    } else {
        str_at(evidence, "reason")
    };
    if !reason.is_empty() {
        limitations.push(reason.to_string());
    }
}

fn is_readiness_target(target: &str) -> bool {
    target.starts_with("readiness:")
        || target.starts_with("readiness/")
        || target.starts_with("channel_ready:")
        || target.starts_with("channel_ready/")
        || target.starts_with("channel-ready:")
        || target.starts_with("channel-ready/")
}

struct Readiness {
    rows: Vec<Value>,
    channel_ready_count: usize,
    failed_count: usize,
    channel_ready_failure_count: usize,
    representative_error: String,
    provider_rows: Vec<Value>,
    provider_attempt_count: usize,
    provider_degraded_count: usize,
    provider_reason: String,
}

fn readiness_evidence(state: &Arc<DaemonState>, channel_h: &str) -> Readiness {
    let mut rows = state
        .session_start
        .lock()
        .expect("session_start mutex poisoned")
        .state_rows()
        .into_iter()
        .filter(|row| row.channel_h == channel_h)
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| a.session_id.cmp(&b.session_id));

    let channel_ready_count = rows
        .iter()
        .filter(|row| row.has_channel_ready_intent)
        .count();
    let failed_count = rows
        .iter()
        .filter(|row| row.action == "RecordFailed")
        .count();
    let channel_ready_failure_count = rows
        .iter()
        .filter(|row| {
            row.action == "RecordFailed" && row.failure_stage.as_deref() == Some("channel_ready")
        })
        .count();
    let representative_error = rows
        .iter()
        .find(|row| {
            row.action == "RecordFailed" && row.failure_stage.as_deref() == Some("channel_ready")
        })
        .and_then(|row| row.failure_error.clone())
        .unwrap_or_default();
    let rows = rows
        .into_iter()
        .take(8)
        .map(|row| {
            json!({
                "session_id": row.session_id,
                "action": row.action,
                "channel_h": row.channel_h,
                "has_channel_ready_intent": row.has_channel_ready_intent,
                "has_spawn_intent": row.has_spawn_intent,
                "ensure_subscription": row.ensure_subscription,
                "reassert": row.reassert,
                "failure_stage": row.failure_stage,
                "failure_error": row.failure_error,
            })
        })
        .collect();
    let attempts = state
        .with_store(|s| s.channel_readiness_attempts(channel_h, 8))
        .unwrap_or_default();
    let provider_attempt_count = attempts.len();
    let provider_degraded_count = attempts
        .iter()
        .filter(|row| row.outcome == "degraded")
        .count();
    let provider_reason = attempts
        .iter()
        .find(|row| row.outcome == "degraded")
        .map(|row| row.reason.clone())
        .or_else(|| attempts.first().map(|row| row.reason.clone()))
        .unwrap_or_default();
    let provider_rows = attempts
        .into_iter()
        .map(|row| {
            json!({
                "id": row.id,
                "channel_h": row.channel_h,
                "expect_member": row.expect_member,
                "parent_hint": row.parent_hint,
                "name": row.name,
                "source": row.source,
                "outcome": row.outcome,
                "reason": row.reason,
                "created_at": row.created_at,
            })
        })
        .collect();
    Readiness {
        rows,
        channel_ready_count,
        failed_count,
        channel_ready_failure_count,
        representative_error,
        provider_rows,
        provider_attempt_count,
        provider_degraded_count,
        provider_reason,
    }
}

fn readiness_summary(found: bool, membership_snapshot: bool, readiness: &Readiness) -> String {
    if found && membership_snapshot {
        return "relay metadata and membership snapshots are hydrated".to_string();
    }
    if readiness.channel_ready_failure_count > 0 {
        return format!(
            "{} channel_ready failure(s) recorded for this channel",
            readiness.channel_ready_failure_count
        );
    }
    if readiness.provider_degraded_count > 0 {
        return format!(
            "{} provider readiness attempt(s) degraded for this channel",
            readiness.provider_degraded_count
        );
    }
    if found {
        return "relay metadata is hydrated, but complete membership snapshots are not".to_string();
    }
    if readiness.channel_ready_count > 0 {
        return "channel readiness was requested, but relay metadata is not materialized"
            .to_string();
    }
    "no relay metadata or session_start readiness attempt is recorded for this channel".to_string()
}

fn readiness_reason(found: bool, membership_snapshot: bool, readiness: &Readiness) -> String {
    if found && membership_snapshot {
        String::new()
    } else if !readiness.representative_error.is_empty() {
        readiness.representative_error.clone()
    } else if !readiness.provider_reason.is_empty() && readiness.provider_degraded_count > 0 {
        readiness.provider_reason.clone()
    } else if found {
        READINESS_LIMITATION.to_string()
    } else if readiness.channel_ready_count > 0 {
        "session_start planned channel_ready, but no relay kind:39000 row is materialized"
            .to_string()
    } else {
        CHANNEL_LIMITATION.to_string()
    }
}

fn channel_reason(found: bool, membership_snapshot: bool, readiness: &Readiness) -> String {
    if readiness.channel_ready_failure_count > 0 {
        return readiness.representative_error.clone();
    }
    if readiness.provider_degraded_count > 0 && !readiness.provider_reason.is_empty() {
        return readiness.provider_reason.clone();
    }
    if readiness.provider_attempt_count > 0 {
        return "provider readiness attempts are recorded; inspect provider_attempt:<id> for the provisioning trace".to_string();
    }
    if found && membership_snapshot {
        return String::new();
    }
    if found {
        return READINESS_LIMITATION.to_string();
    }
    if readiness.channel_ready_count > 0 {
        return "session_start planned channel_ready, but no relay kind:39000 row is materialized"
            .to_string();
    }
    CHANNEL_LIMITATION.to_string()
}

fn str_at<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn bool_at(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn int_at(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}
