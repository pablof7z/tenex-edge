//! Target parsing for `probe validate`.

use super::super::SURFACES;
use crate::explain::Handle;
use serde_json::{json, Value};

pub(super) fn empty_handle_evidence(target: Option<&str>) -> Option<Value> {
    let target = target?;
    const PREFIXES: &[(&str, &str)] = &[
        ("agent:", "agent slug"),
        ("agent/", "agent slug"),
        ("alias:", "session alias"),
        ("alias/", "session alias"),
        ("admin:", "channel admin relation"),
        ("admin/", "channel admin relation"),
        ("awareness:", "awareness channel"),
        ("awareness/", "awareness channel"),
        ("backend:", "backend label"),
        ("backend/", "backend label"),
        ("who:", "awareness channel"),
        ("who/", "awareness channel"),
        ("capsule:", "replay capsule id"),
        ("channel:", "channel id"),
        ("channel/", "channel id"),
        ("channel_ready:", "channel id"),
        ("channel_ready/", "channel id"),
        ("channel-ready:", "channel id"),
        ("channel-ready/", "channel id"),
        ("commit:", "commit ledger row id"),
        ("commit/", "commit ledger row id"),
        ("delivery:", "message recipient edge"),
        ("delivery/", "message recipient edge"),
        ("event/", "event id"),
        ("inbox:", "inbound event id"),
        ("inbox/", "inbound event id"),
        ("identity:", "identity pubkey"),
        ("identity/", "identity pubkey"),
        ("id:", "lookup value"),
        ("id/", "lookup value"),
        ("joined:", "joined session"),
        ("joined/", "joined session"),
        ("lookup:", "lookup value"),
        ("lookup/", "lookup value"),
        ("find:", "lookup value"),
        ("find/", "lookup value"),
        ("message:", "message id"),
        ("message/", "message resource"),
        ("member:", "channel membership relation"),
        ("member/", "channel membership relation"),
        ("membership:", "channel membership relation"),
        ("membership/", "channel membership relation"),
        ("membership_snapshot:", "channel membership snapshot"),
        ("membership_snapshot/", "channel membership snapshot"),
        ("membership-snapshot:", "channel membership snapshot"),
        ("membership-snapshot/", "channel membership snapshot"),
        ("msg:", "message id"),
        ("msg/", "message resource"),
        ("planner:", "planner label"),
        ("planner/", "planner label"),
        ("profile:", "profile pubkey"),
        ("profile/", "profile pubkey"),
        ("provider_attempt:", "provider readiness attempt id"),
        ("provider_attempt/", "provider readiness attempt id"),
        ("provider-attempt:", "provider readiness attempt id"),
        ("provider-attempt/", "provider readiness attempt id"),
        ("project:", "project root channel"),
        ("project/", "project root channel"),
        ("project_root:", "project root channel"),
        ("project_root/", "project root channel"),
        ("project-root:", "project root channel"),
        ("project-root/", "project root channel"),
        ("pubkey:", "pubkey"),
        ("pubkey/", "pubkey"),
        ("quarantine:", "quarantined event id"),
        ("quarantine/", "quarantined event id"),
        ("receipt:", "receipt id"),
        ("recipient:", "message recipient edge"),
        ("recipient/", "message recipient edge"),
        ("roster:", "channel membership snapshot"),
        ("roster/", "channel membership snapshot"),
        ("readiness:", "channel id"),
        ("readiness/", "channel id"),
        ("readiness_attempt:", "provider readiness attempt id"),
        ("readiness_attempt/", "provider readiness attempt id"),
        ("readiness-attempt:", "provider readiness attempt id"),
        ("readiness-attempt/", "provider readiness attempt id"),
        ("sub:", "subscription channel"),
        ("sub/", "subscription resource"),
        ("table:", "durable table name"),
        ("table/", "durable table name"),
        ("status:", "status session"),
        ("status/", "status resource"),
        ("turn:", "turn session"),
        ("turn/", "turn resource"),
        ("turn_lifecycle:", "turn lifecycle session"),
        ("turn_lifecycle/", "turn lifecycle resource"),
        ("trellis_commit:", "commit ledger row id"),
        ("trellis_commit/", "commit ledger row id"),
        ("ledger:", "durable table name"),
        ("ledger/", "durable table name"),
        ("cursor:", "cursor session"),
        ("cursor/", "cursor resource"),
        ("cur:", "cursor session"),
        ("cur/", "cursor resource"),
        ("outbox:", "outbox local id"),
        ("outbox/", "outbox resource"),
        ("session_channel:", "joined session"),
        ("session_channel/", "joined session"),
        ("session-channel:", "joined session"),
        ("session-channel/", "joined session"),
        ("session_start:", "session_start session"),
        ("session_start/", "session_start resource"),
        ("watch:", "session_watch session"),
        ("watch/", "session_watch resource"),
        ("session_watch:", "session_watch session"),
        ("session_watch/", "session_watch resource"),
        ("session-watch/", "session_watch resource"),
        ("state:", "surface name"),
        ("state/", "surface name"),
        ("hook:", "hook_context session"),
        ("hook/", "hook_context resource"),
        ("hook_context:", "hook_context session"),
        ("hook_context/", "hook_context resource"),
        ("harness_session:", "harness session alias"),
        ("harness_session/", "harness session alias"),
        ("harness-session:", "harness session alias"),
        ("harness-session/", "harness session alias"),
        ("resume:", "resume alias"),
        ("resume/", "resume alias"),
        ("tmux_pane:", "tmux pane alias"),
        ("tmux_pane/", "tmux pane alias"),
        ("tmux-pane:", "tmux pane alias"),
        ("tmux-pane/", "tmux pane alias"),
        ("watch_pid:", "watch pid alias"),
        ("watch_pid/", "watch pid alias"),
        ("watch-pid:", "watch pid alias"),
        ("watch-pid/", "watch pid alias"),
        ("work_root:", "project root channel"),
        ("work_root/", "project root channel"),
    ];
    PREFIXES.iter().find_map(|(prefix, label)| {
        target.strip_prefix(prefix).and_then(|rest| {
            let id = rest.split('/').next().unwrap_or(rest);
            id.is_empty().then(|| {
                json!({
                    "target": target,
                    "supported": false,
                    "valid": false,
                    "kind": "empty_handle",
                    "summary": format!("target `{target}` is missing a {label}"),
                    "reason": format!("target `{target}` must include a non-empty {label} after `{prefix}`"),
                })
            })
        })
    })
}

pub(super) fn explain_handle_parse_error(target: Option<&str>) -> Option<Value> {
    let target = target?;
    let (scheme, _) = target.split_once(':')?;
    if !matches!(scheme, "event" | "llm" | "session" | "hook" | "txn" | "sub") {
        return None;
    }
    match crate::explain::parse_handle(target) {
        Ok(handle) => invalid_explain_handle(target, &handle),
        Err(e) => Some(invalid_explain_handle_evidence(target, e.to_string())),
    }
}

pub(super) fn malformed_capsule_target_evidence(target: Option<&str>) -> Option<Value> {
    let target = target?;
    let Some(id) = target.strip_prefix("capsule:") else {
        return None;
    };
    if id.parse::<i64>().is_ok() {
        return None;
    }
    Some(json!({
        "target": target,
        "supported": false,
        "valid": false,
        "kind": "invalid_capsule",
        "summary": format!("target `{target}` is not a valid replay capsule target"),
        "reason": "replay capsule id must be an integer",
    }))
}

pub(super) fn malformed_probe_handle_evidence(target: Option<&str>) -> Option<Value> {
    let target = target?;
    let Some((label, rest)) = target
        .strip_prefix("outbox:")
        .map(|rest| ("outbox", rest))
        .or_else(|| {
            target
                .strip_prefix("receipt:")
                .map(|rest| ("receipt", rest))
        })
        .or_else(|| target.strip_prefix("commit:").map(|rest| ("commit", rest)))
        .or_else(|| {
            target
                .strip_prefix("trellis_commit:")
                .map(|rest| ("commit", rest))
        })
        .or_else(|| {
            target
                .strip_prefix("readiness_attempt:")
                .map(|rest| ("readiness_attempt", rest))
        })
        .or_else(|| {
            target
                .strip_prefix("readiness-attempt:")
                .map(|rest| ("readiness_attempt", rest))
        })
        .or_else(|| {
            target
                .strip_prefix("provider_attempt:")
                .map(|rest| ("readiness_attempt", rest))
        })
        .or_else(|| {
            target
                .strip_prefix("provider-attempt:")
                .map(|rest| ("readiness_attempt", rest))
        })
    else {
        return None;
    };
    if rest.parse::<i64>().is_ok() {
        return None;
    }
    Some(json!({
        "target": target,
        "supported": false,
        "valid": false,
        "kind": "invalid_probe_handle",
        "summary": format!("target `{target}` is not a valid {label} probe handle"),
        "reason": format!("{label} probe handles must use an integer local id"),
    }))
}

pub(super) fn unsupported_target_evidence(
    target: Option<&str>,
    surface: Option<&str>,
    handle: Option<&str>,
    capsule: Option<&str>,
    explain: bool,
    cause_label: bool,
    target_specific: bool,
) -> Option<Value> {
    let target = target?;
    if surface.is_some()
        || handle.is_some()
        || capsule.is_some()
        || explain
        || cause_label
        || target_specific
    {
        return None;
    }
    Some(json!({
        "target": target,
        "supported": false,
        "kind": "unknown_target",
        "summary": format!("target `{target}` is not a known validation target"),
        "reason": "target must be a surface, probe handle, visible Trellis resource path, explain handle, table/ledger target, channel/readiness/readiness_attempt/awareness/message/recipient/project/membership/membership_snapshot/joined/quarantine target, commit target, or `capsule:<id>`",
    }))
}

pub(super) fn awareness_target(target: &str) -> Option<&str> {
    if matches!(target, "awareness" | "who") {
        return Some("");
    }
    target
        .strip_prefix("awareness:")
        .or_else(|| target.strip_prefix("awareness/"))
        .or_else(|| target.strip_prefix("who:"))
        .or_else(|| target.strip_prefix("who/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn channel_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("channel:")
        .or_else(|| target.strip_prefix("channel/"))
        .or_else(|| target.strip_prefix("readiness:"))
        .or_else(|| target.strip_prefix("readiness/"))
        .or_else(|| target.strip_prefix("channel_ready:"))
        .or_else(|| target.strip_prefix("channel_ready/"))
        .or_else(|| target.strip_prefix("channel-ready:"))
        .or_else(|| target.strip_prefix("channel-ready/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn optional_str<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

pub(super) fn surface_target(target: &str) -> Option<&str> {
    let surface = target
        .strip_prefix("state:")
        .or_else(|| target.strip_prefix("state/"))
        .unwrap_or(target);
    SURFACES.iter().copied().find(|known| *known == surface)
}

pub(super) fn surface_from_probe_handle(target: &str) -> Option<&'static str> {
    if target.starts_with("status:") || target.starts_with("status/") {
        Some("status")
    } else if target.starts_with("sub:") || target.starts_with("sub/") {
        Some("subscriptions")
    } else if target.starts_with("hook:")
        || target.starts_with("hook/")
        || target.starts_with("hook_context:")
        || target.starts_with("hook_context/")
    {
        Some("hook_context")
    } else if target.starts_with("turn:")
        || target.starts_with("turn/")
        || target.starts_with("turn_lifecycle:")
        || target.starts_with("turn_lifecycle/")
    {
        Some("turn_lifecycle")
    } else if target.starts_with("cursor:")
        || target.starts_with("cursor/")
        || target.starts_with("cur:")
        || target.starts_with("cur/")
    {
        Some("cursor")
    } else if target.starts_with("outbox:") || target.starts_with("outbox/") {
        Some("outbox")
    } else if target.starts_with("session_start:") || target.starts_with("session_start/") {
        Some("session_start")
    } else if target.starts_with("watch:")
        || target.starts_with("watch/")
        || target.starts_with("session_watch:")
        || target.starts_with("session_watch/")
        || target.starts_with("session-watch/")
    {
        Some("session_watch")
    } else {
        None
    }
}

pub(super) fn surface_from_explain_handle(handle: &Handle) -> Option<&str> {
    match handle {
        Handle::Txn { surface, .. } => Some(surface.as_str()),
        Handle::Hook { .. } => Some("hook_context"),
        Handle::Session { .. } => Some("status"),
        Handle::Sub { .. } => Some("subscriptions"),
        Handle::Event(_) | Handle::Llm(_) => None,
    }
}

pub(super) fn handle_target(target: &str) -> Option<&str> {
    const PREFIXES: [&str; 25] = [
        "sub:",
        "sub/",
        "status:",
        "status/",
        "turn:",
        "turn/",
        "turn_lifecycle:",
        "turn_lifecycle/",
        "cursor:",
        "cursor/",
        "cur:",
        "cur/",
        "outbox:",
        "outbox/",
        "session_start:",
        "session_start/",
        "watch:",
        "watch/",
        "session_watch:",
        "session_watch/",
        "session-watch/",
        "hook:",
        "hook/",
        "hook_context:",
        "hook_context/",
    ];
    PREFIXES
        .iter()
        .any(|p| target.starts_with(p))
        .then_some(target)
}

pub(super) fn capsule_target<'a>(params: &'a Value, target: Option<&'a str>) -> Option<&'a str> {
    optional_str(params, "capsule").or_else(|| target.and_then(|t| t.strip_prefix("capsule:")))
}

fn invalid_explain_handle(target: &str, handle: &Handle) -> Option<Value> {
    match handle {
        Handle::Event(id) if id.is_empty() => Some(invalid_explain_handle_evidence(
            target,
            "event handle id must be non-empty",
        )),
        Handle::Session { id, .. } if id.is_empty() => Some(invalid_explain_handle_evidence(
            target,
            "session handle id must be non-empty",
        )),
        Handle::Hook { id, .. } if id.is_empty() => Some(invalid_explain_handle_evidence(
            target,
            "hook handle id must be non-empty",
        )),
        Handle::Txn { surface, .. } if surface.is_empty() => Some(invalid_explain_handle_evidence(
            target,
            "txn handle surface must be non-empty",
        )),
        Handle::Txn { surface, .. } if !SURFACES.contains(&surface.as_str()) => {
            Some(invalid_explain_handle_evidence(
                target,
                format!("txn handle surface `{surface}` is not a known validation surface"),
            ))
        }
        _ => None,
    }
}

fn invalid_explain_handle_evidence(target: &str, reason: impl Into<String>) -> Value {
    json!({
        "target": target,
        "supported": false,
        "valid": false,
        "kind": "invalid_explain_handle",
        "summary": format!("target `{target}` is not a valid explain handle"),
        "reason": reason.into(),
    })
}
