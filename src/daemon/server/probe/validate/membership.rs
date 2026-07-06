//! Channel membership/admin relation validation.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

mod snapshot;
mod target;

pub(super) use snapshot::{
    membership_snapshot_evidence, membership_snapshot_target, push_membership_snapshot_check,
};
pub(super) use target::{membership_target, MembershipTarget};

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
        "summary": target::summary(&parsed.channel_h, &parsed.pubkey, parsed.require_admin, role, row_found, snapshot),
        "reason": target::reason(row_found, parsed.require_admin, role, channel.is_some(), snapshot),
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
