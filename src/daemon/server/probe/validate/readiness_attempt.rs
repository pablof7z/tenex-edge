//! Direct validation for host/provider channel readiness attempt rows.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn readiness_attempt_target(target: &str) -> Option<i64> {
    target
        .strip_prefix("readiness_attempt:")
        .or_else(|| target.strip_prefix("readiness_attempt/"))
        .or_else(|| target.strip_prefix("readiness-attempt:"))
        .or_else(|| target.strip_prefix("readiness-attempt/"))
        .or_else(|| target.strip_prefix("provider_attempt:"))
        .or_else(|| target.strip_prefix("provider_attempt/"))
        .or_else(|| target.strip_prefix("provider-attempt:"))
        .or_else(|| target.strip_prefix("provider-attempt/"))
        .and_then(|rest| rest.split('/').next())
        .and_then(|id| id.parse::<i64>().ok())
}

pub(super) fn readiness_attempt_evidence(state: &Arc<DaemonState>, target: &str, id: i64) -> Value {
    let result = state.with_store(|store| {
        let Some(row) = store.channel_readiness_attempt(id)? else {
            return Ok::<_, anyhow::Error>((None, None, Vec::new(), false));
        };
        let channel = store.get_channel(&row.channel_h)?;
        let members = store.list_channel_members(&row.channel_h)?;
        let snapshot = store.has_channel_membership_snapshot(&row.channel_h)?;
        Ok::<_, anyhow::Error>((Some(row), channel, members, snapshot))
    });
    let (row, channel, members, membership_snapshot) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "id": id,
                "kind": "readiness_attempt",
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": format!("readiness attempt `{id}` evidence failed"),
                "reason": e.to_string(),
            });
        }
    };
    let Some(row) = row else {
        return json!({
            "target": target,
            "id": id,
            "kind": "readiness_attempt",
            "supported": true,
            "found": false,
            "summary": format!("readiness attempt `{id}` is not recorded"),
            "reason": "no channel_readiness_attempts row matched this id",
        });
    };

    let expected_member = row.expect_member.as_str();
    let expected_member_found = !expected_member.is_empty()
        && members
            .iter()
            .any(|member| member.pubkey == expected_member);
    let expected_member_role = members
        .iter()
        .find(|member| member.pubkey == expected_member)
        .map(|member| member.role.as_str())
        .unwrap_or("");
    let degraded = degraded_outcome(&row.outcome);
    let ready = ready_outcome(&row.outcome);
    let current_ready = channel.is_some()
        && (expected_member.is_empty() || expected_member_found || !membership_snapshot);

    json!({
        "target": target,
        "id": row.id,
        "kind": "readiness_attempt",
        "supported": true,
        "found": true,
        "channel_h": row.channel_h,
        "expect_member": row.expect_member,
        "parent_hint": row.parent_hint,
        "name": row.name,
        "source": row.source,
        "outcome": row.outcome,
        "attempt_reason": row.reason,
        "created_at": row.created_at,
        "channel_found": channel.is_some(),
        "channel_name": channel.as_ref().map(|c| c.name.as_str()).unwrap_or(""),
        "membership_snapshot": membership_snapshot,
        "member_count": members.len(),
        "admin_count": members.iter().filter(|member| member.role == "admin").count(),
        "expected_member_found": expected_member_found,
        "expected_member_role": expected_member_role,
        "ready_outcome": ready,
        "degraded_outcome": degraded,
        "current_ready": current_ready,
        "summary": summary(row.id, &row.channel_h, &row.outcome, degraded, ready, current_ready),
        "reason": reason(&row, channel.is_some(), membership_snapshot, expected_member_found),
    })
}

pub(super) fn push_readiness_attempt_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty()
        || bool_at(evidence, "degraded_outcome")
        || (bool_at(evidence, "ready_outcome")
            && bool_at(evidence, "membership_snapshot")
            && !str_at(evidence, "expect_member").is_empty()
            && !bool_at(evidence, "expected_member_found"))
    {
        "failed"
    } else if bool_at(evidence, "ready_outcome") && bool_at(evidence, "current_ready") {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "readiness_attempt",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    } else if status == "passed" {
        limitations.push(
            "readiness attempt proves a local host/provider decision; relay state remains the channel source of truth"
                .to_string(),
        );
    }
}

fn summary(
    id: i64,
    channel_h: &str,
    outcome: &str,
    degraded: bool,
    ready: bool,
    current_ready: bool,
) -> String {
    if degraded {
        return format!("readiness attempt `{id}` degraded for channel `{channel_h}`");
    }
    if ready && current_ready {
        return format!("readiness attempt `{id}` verified channel `{channel_h}`");
    }
    if ready {
        return format!(
            "readiness attempt `{id}` recorded ready for channel `{channel_h}`, but current readiness is not proven"
        );
    }
    format!("readiness attempt `{id}` has outcome `{outcome}` for channel `{channel_h}`")
}

fn reason(
    row: &crate::state::ChannelReadinessAttempt,
    channel_found: bool,
    membership_snapshot: bool,
    expected_member_found: bool,
) -> String {
    if degraded_outcome(&row.outcome) {
        return row.reason.clone();
    }
    if !ready_outcome(&row.outcome) {
        return "readiness attempt outcome is not a recognized ready/degraded state".into();
    }
    if !channel_found {
        return "attempt recorded ready, but no relay channel metadata is currently materialized"
            .into();
    }
    if !row.expect_member.is_empty() && membership_snapshot && !expected_member_found {
        return "attempt expected a member pubkey that is absent from the hydrated membership snapshot"
            .into();
    }
    if !row.expect_member.is_empty() && !membership_snapshot {
        return "attempt recorded ready, but complete membership snapshots are not hydrated".into();
    }
    row.reason.clone()
}

fn ready_outcome(outcome: &str) -> bool {
    matches!(outcome, "ready" | "ok" | "succeeded" | "verified")
}

fn degraded_outcome(outcome: &str) -> bool {
    matches!(outcome, "degraded" | "failed" | "error")
}
