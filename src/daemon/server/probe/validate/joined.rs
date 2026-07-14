//! Joined-channel validation for local session listening scope.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

pub(super) struct JoinedTarget {
    session_id: String,
    channel_h: Option<String>,
}

pub(super) fn joined_target(target: &str) -> Option<JoinedTarget> {
    if let Some(rest) = target
        .strip_prefix("joined:")
        .or_else(|| target.strip_prefix("session_channel:"))
        .or_else(|| target.strip_prefix("session-channel:"))
    {
        return split_colon_target(rest);
    }
    let rest = target
        .strip_prefix("joined/")
        .or_else(|| target.strip_prefix("session_channel/"))
        .or_else(|| target.strip_prefix("session-channel/"))?;
    let (session_id, channel_h) = match rest.split_once('/') {
        Some((session, channel)) => (session, Some(channel)),
        None => (rest, None),
    };
    (!session_id.trim().is_empty()).then(|| JoinedTarget {
        session_id: session_id.to_string(),
        channel_h: channel_h
            .filter(|channel| !channel.trim().is_empty())
            .map(str::to_string),
    })
}

pub(super) fn joined_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    parsed: &JoinedTarget,
) -> Value {
    let result = state.with_store(|store| {
        let session = store.get_session(&parsed.session_id)?;
        let joined = store.list_session_joined_channels(&parsed.session_id)?;
        let rows = joined
            .iter()
            .map(|(channel_h, joined_at)| {
                let channel = store.get_channel(channel_h)?;
                Ok::<_, anyhow::Error>((channel_h.clone(), *joined_at, channel))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok::<_, anyhow::Error>((session, rows))
    });
    let (session, rows) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "session_id": parsed.session_id,
                "channel_h": parsed.channel_h,
                "kind": "joined",
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": "joined-channel evidence could not read local state",
                "reason": e.to_string(),
            });
        }
    };
    let Some(session) = session else {
        return json!({
            "target": target,
            "session_id": parsed.session_id,
            "channel_h": parsed.channel_h,
            "kind": "joined",
            "supported": true,
            "found": false,
            "summary": format!("session `{}` has no local row", parsed.session_id),
            "reason": "no local session row exists for this session id or alias",
        });
    };

    let owner = format!("session-{}", session.pubkey);
    let subs = state.subs.lock().expect("subs mutex poisoned").state_rows();
    let row_values = rows
        .iter()
        .map(|(channel_h, joined_at, channel)| {
            row_json(channel_h, *joined_at, channel.as_ref(), &subs, &owner)
        })
        .collect::<Vec<_>>();
    let joined_channels = rows
        .iter()
        .map(|(channel_h, _, _)| channel_h.as_str())
        .collect::<BTreeSet<_>>();
    let requested_joined = parsed
        .channel_h
        .as_ref()
        .is_none_or(|channel| joined_channels.contains(channel.as_str()));
    let missing_subscription_count = row_values
        .iter()
        .filter(|row| !bool_at(row, "sub_h_owned") || !bool_at(row, "sub_d_owned"))
        .count();
    let missing_channel_count = row_values
        .iter()
        .filter(|row| !bool_at(row, "channel_found"))
        .count();
    let found = !rows.is_empty() && requested_joined;
    let ok =
        session.alive && found && missing_subscription_count == 0 && missing_channel_count == 0;

    json!({
        "target": target,
        "pubkey": session.pubkey,
        "requested_session_id": parsed.session_id,
        "channel_h": parsed.channel_h,
        "kind": "joined",
        "supported": true,
        "found": found,
        "session_found": true,
        "session_alive": session.alive,
        "active_channel_h": session.channel_h,
        "joined_count": rows.len(),
        "requested_joined": requested_joined,
        "missing_subscription_count": missing_subscription_count,
        "missing_channel_count": missing_channel_count,
        "rows": row_values,
        "ok": ok,
        "summary": summary(&session.pubkey, parsed.channel_h.as_deref(), rows.len(), requested_joined, session.alive, missing_subscription_count, missing_channel_count),
        "reason": reason(rows.is_empty(), requested_joined, session.alive, missing_subscription_count, missing_channel_count),
    })
}

pub(super) fn push_joined_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty()
        || (bool_at(evidence, "session_alive")
            && bool_at(evidence, "found")
            && !bool_at(evidence, "ok"))
    {
        "failed"
    } else if bool_at(evidence, "ok") {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "joined_channels",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn split_colon_target(rest: &str) -> Option<JoinedTarget> {
    if rest.trim().is_empty() {
        return None;
    }
    match rest.split_once(':') {
        Some((session, channel)) if !session.trim().is_empty() && !channel.trim().is_empty() => {
            Some(JoinedTarget {
                session_id: session.to_string(),
                channel_h: Some(channel.to_string()),
            })
        }
        Some((session, _)) if !session.trim().is_empty() => Some(JoinedTarget {
            session_id: session.to_string(),
            channel_h: None,
        }),
        Some(_) => None,
        None => Some(JoinedTarget {
            session_id: rest.to_string(),
            channel_h: None,
        }),
    }
}

fn row_json(
    channel_h: &str,
    joined_at: u64,
    channel: Option<&crate::state::Channel>,
    subs: &[crate::reconcile::subscriptions::probe::SubStateRow],
    owner: &str,
) -> Value {
    json!({
        "channel_h": channel_h,
        "joined_at": joined_at,
        "channel_found": channel.is_some(),
        "channel_name": channel.map(|c| c.name.as_str()).unwrap_or(""),
        "channel_parent": channel.map(|c| c.parent.as_str()).unwrap_or(""),
        "sub_h_owned": owner_has_subscription(subs, &format!("sub/h/{channel_h}"), owner),
        "sub_d_owned": owner_has_subscription(subs, &format!("sub/d/{channel_h}"), owner),
    })
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

fn summary(
    session_id: &str,
    requested_channel: Option<&str>,
    joined_count: usize,
    requested_joined: bool,
    alive: bool,
    missing_subscriptions: usize,
    missing_channels: usize,
) -> String {
    if let Some(channel) = requested_channel {
        if requested_joined && alive && missing_subscriptions == 0 && missing_channels == 0 {
            return format!(
                "session `{session_id}` is joined to `{channel}` with subscription coverage"
            );
        }
        if !requested_joined {
            return format!("session `{session_id}` is not joined to `{channel}`");
        }
    }
    if joined_count == 0 {
        return format!("session `{session_id}` has no joined channel rows");
    }
    if !alive {
        return format!(
            "session `{session_id}` is not alive; joined channel coverage is historical"
        );
    }
    if missing_channels > 0 {
        return format!(
            "session `{session_id}` has {missing_channels} joined channel(s) without relay metadata"
        );
    }
    if missing_subscriptions > 0 {
        return format!(
            "session `{session_id}` has {missing_subscriptions} joined channel(s) without subscription coverage"
        );
    }
    format!(
        "session `{session_id}` has {joined_count} joined channel(s) with subscription coverage"
    )
}

fn reason(
    no_rows: bool,
    requested_joined: bool,
    alive: bool,
    missing_subscriptions: usize,
    missing_channels: usize,
) -> &'static str {
    if no_rows {
        "no joined channel rows exist for this session"
    } else if !requested_joined {
        "requested channel is not in the session's joined-channel set"
    } else if !alive {
        "dead sessions retain historical joined-channel rows; live coverage is not proven"
    } else if missing_channels > 0 {
        "one or more joined channels lacks relay channel metadata"
    } else if missing_subscriptions > 0 {
        "one or more joined channels is missing sub/h or sub/d subscription coverage"
    } else {
        ""
    }
}
