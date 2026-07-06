//! Channel membership/admin relation validation.

use super::report::{bool_at, int_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) struct MembershipTarget {
    channel_h: String,
    pubkey: String,
    require_admin: bool,
}

pub(super) struct MembershipSnapshotTarget {
    channel_h: String,
}

pub(super) fn membership_target(target: &str) -> Option<MembershipTarget> {
    colon_target(target, "member:", false)
        .or_else(|| colon_target(target, "membership:", false))
        .or_else(|| colon_target(target, "admin:", true))
        .or_else(|| path_target(target, "member/", false))
        .or_else(|| path_target(target, "membership/", false))
        .or_else(|| path_target(target, "admin/", true))
}

pub(super) fn membership_snapshot_target(target: &str) -> Option<MembershipSnapshotTarget> {
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

pub(super) fn membership_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    parsed: &MembershipTarget,
) -> Value {
    let result = state.with_store(|store| {
        let channel = store.get_channel(&parsed.channel_h)?;
        let members = store.list_channel_members(&parsed.channel_h)?;
        let row = members
            .iter()
            .find(|member| member.pubkey == parsed.pubkey)
            .cloned();
        let membership_snapshot = store.has_channel_membership_snapshot(&parsed.channel_h)?;
        let profile = store.get_profile(&parsed.pubkey)?;
        let identity = store.get_identity(&parsed.pubkey)?;
        let session = match identity.as_ref().filter(|row| !row.session_id.is_empty()) {
            Some(row) => store.get_session(&row.session_id)?,
            None => None,
        };
        Ok::<_, anyhow::Error>((
            channel,
            row,
            members.len(),
            members.iter().filter(|m| m.role == "admin").count(),
            membership_snapshot,
            profile,
            identity,
            session,
        ))
    });
    let (channel, row, member_count, admin_count, snapshot, profile, identity, session) =
        match result {
            Ok(v) => v,
            Err(e) => {
                return json!({
                    "target": target,
                    "kind": "membership",
                    "channel_h": parsed.channel_h,
                    "pubkey": parsed.pubkey,
                    "require_admin": parsed.require_admin,
                    "supported": true,
                    "found": false,
                    "error": e.to_string(),
                    "summary": "membership evidence could not read durable state",
                    "reason": e.to_string(),
                });
            }
        };
    let row_found = row.is_some();
    let role = row.as_ref().map(|m| m.role.as_str()).unwrap_or("");
    let role_satisfies = row_found && (!parsed.require_admin || role == "admin");
    let ok = role_satisfies;

    json!({
        "target": target,
        "kind": "membership",
        "channel_h": parsed.channel_h,
        "pubkey": parsed.pubkey,
        "require_admin": parsed.require_admin,
        "supported": true,
        "found": row_found,
        "ok": ok,
        "channel_found": channel.is_some(),
        "channel_name": channel.as_ref().map(|c| c.name.as_str()).unwrap_or(""),
        "membership_snapshot": snapshot,
        "member_count": member_count,
        "admin_count": admin_count,
        "role": role,
        "updated_at": row.as_ref().map(|m| m.updated_at).unwrap_or(0),
        "profile_found": profile.is_some(),
        "profile_slug": profile.as_ref().map(|p| p.slug.as_str()).unwrap_or(""),
        "profile_name": profile.as_ref().map(|p| p.name.as_str()).unwrap_or(""),
        "identity_found": identity.is_some(),
        "identity_alive": identity.as_ref().is_some_and(|i| i.alive),
        "identity_session_id": identity.as_ref().map(|i| i.session_id.as_str()).unwrap_or(""),
        "session_found": session.is_some(),
        "session_alive": session.as_ref().is_some_and(|s| s.alive),
        "summary": summary(&parsed.channel_h, &parsed.pubkey, parsed.require_admin, role, row_found, snapshot),
        "reason": reason(row_found, parsed.require_admin, role, channel.is_some(), snapshot),
    })
}

pub(super) fn membership_snapshot_evidence(
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

pub(super) fn push_membership_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let row_found = bool_at(evidence, "found");
    let require_admin = bool_at(evidence, "require_admin");
    let role = str_at(evidence, "role");
    let status = if !str_at(evidence, "error").is_empty()
        || (row_found && require_admin && role != "admin")
        || (!row_found && bool_at(evidence, "membership_snapshot"))
    {
        "failed"
    } else if bool_at(evidence, "ok") {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "membership",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    } else if status == "passed" && !bool_at(evidence, "membership_snapshot") {
        limitations.push(
            "membership row exists, but complete admin/member snapshots are not hydrated"
                .to_string(),
        );
    } else if status == "passed" && !bool_at(evidence, "channel_found") {
        limitations.push("membership row exists, but channel metadata is not materialized".into());
    }
}

pub(super) fn push_membership_snapshot_check(
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

fn colon_target(target: &str, prefix: &str, require_admin: bool) -> Option<MembershipTarget> {
    let rest = target.strip_prefix(prefix)?;
    let (channel_h, pubkey) = rest.split_once(':')?;
    build_target(channel_h, pubkey, require_admin)
}

fn path_target(target: &str, prefix: &str, require_admin: bool) -> Option<MembershipTarget> {
    let rest = target.strip_prefix(prefix)?;
    let (channel_h, pubkey) = rest.split_once('/')?;
    build_target(channel_h, pubkey, require_admin)
}

fn build_target(channel_h: &str, pubkey: &str, require_admin: bool) -> Option<MembershipTarget> {
    (!channel_h.trim().is_empty() && !pubkey.trim().is_empty()).then(|| MembershipTarget {
        channel_h: channel_h.to_string(),
        pubkey: pubkey.to_string(),
        require_admin,
    })
}

fn summary(
    channel_h: &str,
    pubkey: &str,
    require_admin: bool,
    role: &str,
    found: bool,
    snapshot: bool,
) -> String {
    let target_role = if require_admin { "admin" } else { "member" };
    if found && (!require_admin || role == "admin") {
        return format!("pubkey `{pubkey}` is {role} in channel `{channel_h}`");
    }
    if found {
        return format!(
            "pubkey `{pubkey}` is `{role}` in channel `{channel_h}`, not `{target_role}`"
        );
    }
    if snapshot {
        format!("pubkey `{pubkey}` is not in the hydrated `{channel_h}` membership snapshot")
    } else {
        format!("pubkey `{pubkey}` is not proven in channel `{channel_h}`")
    }
}

fn reason(
    found: bool,
    require_admin: bool,
    role: &str,
    channel_found: bool,
    snapshot: bool,
) -> &'static str {
    if found && require_admin && role != "admin" {
        "membership row exists, but it is not an admin role"
    } else if found && !channel_found {
        "membership row exists, but channel metadata is not materialized"
    } else if found && !snapshot {
        "membership row exists, but complete admin/member snapshots are not hydrated"
    } else if !found && snapshot {
        "hydrated channel membership snapshot does not contain this pubkey"
    } else if !found && !channel_found {
        "channel metadata is not materialized and no membership row matched this pubkey"
    } else if !found {
        "membership snapshot is not fully hydrated, so absence is not proven"
    } else {
        ""
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
