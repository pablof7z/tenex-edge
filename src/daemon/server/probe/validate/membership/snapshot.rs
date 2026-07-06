use super::super::report::{bool_at, int_at, str_at};
use super::super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(in crate::daemon::server::probe::validate) struct MembershipSnapshotTarget {
    channel_h: String,
}

pub(in crate::daemon::server::probe::validate) fn membership_snapshot_target(
    target: &str,
) -> Option<MembershipSnapshotTarget> {
    target
        .strip_prefix("membership_snapshot:")
        .or_else(|| target.strip_prefix("membership_snapshot/"))
        .or_else(|| target.strip_prefix("membership-snapshot:"))
        .or_else(|| target.strip_prefix("membership-snapshot/"))
        .or_else(|| target.strip_prefix("roster:"))
        .or_else(|| target.strip_prefix("roster/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.trim().is_empty())
        .map(|channel_h| MembershipSnapshotTarget {
            channel_h: channel_h.to_string(),
        })
}

pub(in crate::daemon::server::probe::validate) fn membership_snapshot_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    parsed: &MembershipSnapshotTarget,
) -> Value {
    let result = state.with_store(|store| {
        let channel = store.get_channel(&parsed.channel_h)?;
        let sets = store.channel_member_sets(&parsed.channel_h)?;
        let members = store.list_channel_members(&parsed.channel_h)?;
        Ok::<_, anyhow::Error>((channel, sets, members))
    });
    let (channel, sets, members) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "kind": "membership_snapshot",
                "channel_h": parsed.channel_h,
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": "membership snapshot evidence could not read durable state",
                "reason": e.to_string(),
            });
        }
    };
    let admin_set = sets.iter().find(|set| set.role == "admin");
    let member_set = sets.iter().find(|set| set.role == "member");
    let admin_count = members
        .iter()
        .filter(|member| member.role == "admin")
        .count();
    let member_count = members.len();
    let snapshot_complete = admin_set.is_some() && member_set.is_some();

    json!({
        "target": target,
        "kind": "membership_snapshot",
        "channel_h": parsed.channel_h,
        "supported": true,
        "found": snapshot_complete,
        "channel_found": channel.is_some(),
        "channel_name": channel.as_ref().map(|c| c.name.as_str()).unwrap_or(""),
        "admin_set_found": admin_set.is_some(),
        "member_set_found": member_set.is_some(),
        "admin_set_updated_at": admin_set.map(|set| set.updated_at).unwrap_or(0),
        "member_set_updated_at": member_set.map(|set| set.updated_at).unwrap_or(0),
        "set_count": sets.len(),
        "sets": sets.iter().map(|set| json!({
            "channel_h": set.channel_h,
            "role": set.role,
            "updated_at": set.updated_at,
        })).collect::<Vec<_>>(),
        "member_count": member_count,
        "admin_count": admin_count,
        "members": members.iter().take(10).map(|member| json!({
            "pubkey": member.pubkey,
            "role": member.role,
            "updated_at": member.updated_at,
        })).collect::<Vec<_>>(),
        "snapshot_complete": snapshot_complete,
        "summary": snapshot_summary(&parsed.channel_h, snapshot_complete, admin_set.is_some(), member_set.is_some(), admin_count),
        "reason": snapshot_reason(snapshot_complete, channel.is_some(), admin_set.is_some(), member_set.is_some(), admin_count),
    })
}

pub(in crate::daemon::server::probe::validate) fn push_membership_snapshot_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty()
        || (bool_at(evidence, "snapshot_complete") && int_at(evidence, "admin_count") == 0)
    {
        "failed"
    } else if bool_at(evidence, "snapshot_complete") {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "membership_snapshot",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    } else if status == "passed" && !bool_at(evidence, "channel_found") {
        limitations
            .push("membership snapshot exists, but channel metadata is not materialized".into());
    }
}

fn snapshot_summary(
    channel_h: &str,
    complete: bool,
    admin_set: bool,
    member_set: bool,
    admin_count: usize,
) -> String {
    if complete && admin_count > 0 {
        return format!("channel `{channel_h}` has hydrated admin and member snapshots");
    }
    if complete {
        return format!("channel `{channel_h}` has hydrated snapshots but no admin row");
    }
    let missing = match (admin_set, member_set) {
        (false, false) => "admin and member snapshots",
        (false, true) => "admin snapshot",
        (true, false) => "member snapshot",
        (true, true) => "snapshot",
    };
    format!("channel `{channel_h}` is missing {missing}")
}

fn snapshot_reason(
    complete: bool,
    channel_found: bool,
    admin_set: bool,
    member_set: bool,
    admin_count: usize,
) -> &'static str {
    if complete && admin_count == 0 {
        "hydrated membership snapshot contains no admin row"
    } else if complete && !channel_found {
        "membership snapshot exists, but channel metadata is not materialized"
    } else if complete {
        ""
    } else if !admin_set && !member_set {
        "neither admin nor member replacement snapshots are hydrated"
    } else if !admin_set {
        "member snapshot is hydrated, but admin replacement snapshot is missing"
    } else {
        "admin snapshot is hydrated, but member replacement snapshot is missing"
    }
}
