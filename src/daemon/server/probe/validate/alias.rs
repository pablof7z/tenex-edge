//! Session alias validation for external-id to canonical-session bindings.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) struct AliasTarget {
    harness: Option<String>,
    kind: String,
    external_id: String,
}

pub(super) fn alias_target(target: &str) -> Option<AliasTarget> {
    if let Some(rest) = target.strip_prefix("alias:") {
        let parts = rest.splitn(3, ':').collect::<Vec<_>>();
        return alias_parts(parts.first()?, parts.get(1)?, parts.get(2)?);
    }
    if let Some(rest) = target.strip_prefix("alias/") {
        let parts = rest.splitn(3, '/').collect::<Vec<_>>();
        return alias_parts(parts.first()?, parts.get(1)?, parts.get(2)?);
    }
    harnessed(target, "harness_session:", "harness_session")
        .or_else(|| harnessed(target, "harness-session:", "harness_session"))
        .or_else(|| harnessed_path(target, "harness_session/", "harness_session"))
        .or_else(|| harnessed_path(target, "harness-session/", "harness_session"))
        .or_else(|| harnessed(target, "resume:", "resume"))
        .or_else(|| harnessed_path(target, "resume/", "resume"))
        .or_else(|| machine_wide(target, "tmux_pane:", "tmux_pane"))
        .or_else(|| machine_wide(target, "tmux-pane:", "tmux_pane"))
        .or_else(|| machine_wide(target, "tmux_pane/", "tmux_pane"))
        .or_else(|| machine_wide(target, "tmux-pane/", "tmux_pane"))
        .or_else(|| machine_wide(target, "watch_pid:", "watch_pid"))
        .or_else(|| machine_wide(target, "watch-pid:", "watch_pid"))
        .or_else(|| machine_wide(target, "watch_pid/", "watch_pid"))
        .or_else(|| machine_wide(target, "watch-pid/", "watch_pid"))
}

pub(super) fn alias_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    parsed: &AliasTarget,
) -> Value {
    let result = state.with_store(|store| {
        let aliases = store.aliases_for_external_id(
            parsed.harness.as_deref(),
            &parsed.kind,
            &parsed.external_id,
        )?;
        let live = store.alive_session_for_alias(
            parsed.harness.as_deref(),
            &parsed.kind,
            &parsed.external_id,
        )?;
        let sessions = aliases
            .iter()
            .map(|alias| store.get_session(&alias.session_id))
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok::<_, anyhow::Error>((aliases, live, sessions))
    });
    let (aliases, live, sessions) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "kind": "alias",
                "alias_kind": parsed.kind.as_str(),
                "external_id": parsed.external_id.as_str(),
                "harness": parsed.harness.as_deref(),
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": "alias evidence could not read durable session aliases",
                "reason": e.to_string(),
            });
        }
    };

    let primary = live.as_ref().or_else(|| sessions.iter().flatten().next());
    let canonical = primary.map(|s| s.session_id.as_str()).unwrap_or("");
    let (status_found, watch_found, sub_h_owned, sub_d_owned) = primary
        .map(|session| live_surface_flags(state, session))
        .unwrap_or((false, false, false, false));
    let ambiguous = parsed.harness.is_none() && aliases.len() > 1;
    let row_missing_session = !aliases.is_empty() && sessions.iter().any(Option::is_none);
    let missing = primary
        .filter(|session| session.alive)
        .map(|session| {
            missing_surfaces(session, status_found, watch_found, sub_h_owned, sub_d_owned)
        })
        .unwrap_or_default();
    let ok = !ambiguous
        && !row_missing_session
        && live.is_some()
        && primary.is_some_and(|session| session.alive)
        && missing.is_empty();

    json!({
        "target": target,
        "kind": "alias",
        "alias_kind": parsed.kind.as_str(),
        "external_id": parsed.external_id.as_str(),
        "harness": parsed.harness.as_deref(),
        "supported": true,
        "found": !aliases.is_empty(),
        "ambiguous": ambiguous,
        "row_count": aliases.len(),
        "rows": aliases.iter().zip(sessions.iter()).take(10).map(alias_json).collect::<Vec<_>>(),
        "resolved_live": live.is_some(),
        "resolved_session_id": canonical,
        "session_found": primary.is_some(),
        "session_alive": primary.is_some_and(|s| s.alive),
        "agent_slug": primary.map(|s| s.agent_slug.as_str()).unwrap_or(""),
        "channel_h": primary.map(|s| s.channel_h.as_str()).unwrap_or(""),
        "status_found": status_found,
        "watch_found": watch_found,
        "sub_h_owned": sub_h_owned,
        "sub_d_owned": sub_d_owned,
        "missing": missing,
        "row_missing_session": row_missing_session,
        "ok": ok,
        "summary": summary(parsed, canonical, aliases.len(), ambiguous, live.is_some(), row_missing_session, ok),
        "reason": reason(aliases.is_empty(), ambiguous, live.is_some(), row_missing_session, ok),
    })
}

pub(super) fn push_alias_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty()
        || bool_at(evidence, "row_missing_session")
        || (bool_at(evidence, "session_alive") && !bool_at(evidence, "ok"))
    {
        "failed"
    } else if bool_at(evidence, "ok") {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "alias",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn alias_parts(harness: &str, kind: &str, external_id: &str) -> Option<AliasTarget> {
    (!harness.trim().is_empty() && !kind.trim().is_empty() && !external_id.trim().is_empty()).then(
        || AliasTarget {
            harness: Some(harness.to_string()),
            kind: normalize_kind(kind).to_string(),
            external_id: external_id.to_string(),
        },
    )
}

fn harnessed(target: &str, prefix: &str, kind: &str) -> Option<AliasTarget> {
    let rest = target.strip_prefix(prefix)?;
    let (harness, external_id) = rest.split_once(':')?;
    alias_parts(harness, kind, external_id)
}

fn harnessed_path(target: &str, prefix: &str, kind: &str) -> Option<AliasTarget> {
    let rest = target.strip_prefix(prefix)?;
    let (harness, external_id) = rest.split_once('/')?;
    alias_parts(harness, kind, external_id)
}

fn machine_wide(target: &str, prefix: &str, kind: &str) -> Option<AliasTarget> {
    let external_id = target.strip_prefix(prefix)?;
    (!external_id.trim().is_empty()).then(|| AliasTarget {
        harness: None,
        kind: kind.to_string(),
        external_id: external_id.to_string(),
    })
}

fn normalize_kind(kind: &str) -> &str {
    match kind {
        "harness-session" => "harness_session",
        "tmux-pane" => "tmux_pane",
        "watch-pid" => "watch_pid",
        other => other,
    }
}

fn live_surface_flags(
    state: &Arc<DaemonState>,
    session: &crate::state::Session,
) -> (bool, bool, bool, bool) {
    let status_found = state
        .status
        .lock()
        .expect("status mutex poisoned")
        .state_rows()
        .into_iter()
        .any(|row| row.session == session.session_id);
    let watch_found = state
        .session_watch
        .lock()
        .expect("session_watch mutex poisoned")
        .state_rows()
        .into_iter()
        .any(|row| row.session == session.session_id);
    let owner = format!("session-{}", session.session_id);
    let subs = state.subs.lock().expect("subs mutex poisoned").state_rows();
    let sub_h_owned =
        owner_has_subscription(&subs, &format!("sub/h/{}", session.channel_h), &owner);
    let sub_d_owned =
        owner_has_subscription(&subs, &format!("sub/d/{}", session.channel_h), &owner);
    (status_found, watch_found, sub_h_owned, sub_d_owned)
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

fn missing_surfaces(
    session: &crate::state::Session,
    status_found: bool,
    watch_found: bool,
    sub_h_owned: bool,
    sub_d_owned: bool,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !status_found {
        missing.push("status");
    }
    if !watch_found {
        missing.push("session_watch");
    }
    if session.channel_h.trim().is_empty() {
        missing.push("active_channel");
    } else {
        if !sub_h_owned {
            missing.push("sub/h");
        }
        if !sub_d_owned {
            missing.push("sub/d");
        }
    }
    missing
}

fn alias_json(
    (alias, session): (&crate::state::SessionAlias, &Option<crate::state::Session>),
) -> Value {
    json!({
        "harness": alias.harness,
        "external_id_kind": alias.external_id_kind,
        "external_id": alias.external_id,
        "session_id": alias.session_id,
        "created_at": alias.created_at,
        "session_found": session.is_some(),
        "session_alive": session.as_ref().is_some_and(|s| s.alive),
        "session_channel_h": session.as_ref().map(|s| s.channel_h.as_str()).unwrap_or(""),
        "agent_slug": session.as_ref().map(|s| s.agent_slug.as_str()).unwrap_or(""),
    })
}

fn summary(
    parsed: &AliasTarget,
    canonical: &str,
    rows: usize,
    ambiguous: bool,
    live_resolved: bool,
    row_missing_session: bool,
    ok: bool,
) -> String {
    let label = alias_label(parsed);
    if ok {
        format!("{label} resolves to live session `{canonical}` with surface evidence")
    } else if rows == 0 {
        format!("{label} has no alias row")
    } else if ambiguous {
        format!("{label} matches multiple alias rows")
    } else if row_missing_session {
        format!("{label} points at a missing canonical session row")
    } else if live_resolved {
        format!("{label} resolves to live session `{canonical}` with missing surface evidence")
    } else {
        format!("{label} does not resolve to a live session")
    }
}

fn reason(
    missing_alias: bool,
    ambiguous: bool,
    live_resolved: bool,
    row_missing_session: bool,
    ok: bool,
) -> &'static str {
    if ok {
        ""
    } else if missing_alias {
        "no session_aliases row matched this external id"
    } else if ambiguous {
        "machine-wide alias lookup matched multiple rows; use alias:<harness>:<kind>:<external_id>"
    } else if row_missing_session {
        "alias row points at a canonical session id that is missing from sessions"
    } else if !live_resolved {
        "alias exists, but it does not resolve to an alive local session"
    } else {
        "alive session resolved by alias is missing status, session_watch, or active-channel subscription evidence"
    }
}

fn alias_label(parsed: &AliasTarget) -> String {
    match parsed.harness.as_deref() {
        Some(harness) => format!(
            "alias `{harness}:{kind}:{external_id}`",
            kind = parsed.kind,
            external_id = parsed.external_id
        ),
        None => format!(
            "alias `{kind}:{external_id}`",
            kind = parsed.kind,
            external_id = parsed.external_id
        ),
    }
}
