use super::super::super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

const CHANNEL_LIMITATION: &str = "channel provisioning is a host/provider side effect; no provider readiness attempt is recorded for this channel";
const READINESS_LIMITATION: &str =
    "channel readiness is advisory unless relay metadata and membership snapshots are hydrated";
pub(super) struct Readiness {
    pub(super) rows: Vec<Value>,
    pub(super) channel_ready_count: usize,
    pub(super) failed_count: usize,
    pub(super) channel_ready_failure_count: usize,
    pub(super) representative_error: String,
    pub(super) provider_rows: Vec<Value>,
    pub(super) provider_attempt_count: usize,
    pub(super) provider_degraded_count: usize,
    pub(super) provider_reason: String,
}

pub(super) fn evidence(state: &Arc<DaemonState>, channel_h: &str) -> Readiness {
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

pub(super) fn summary(found: bool, membership_snapshot: bool, readiness: &Readiness) -> String {
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

pub(super) fn readiness_reason(
    found: bool,
    membership_snapshot: bool,
    readiness: &Readiness,
) -> String {
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

pub(super) fn channel_reason(
    found: bool,
    membership_snapshot: bool,
    readiness: &Readiness,
) -> String {
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
