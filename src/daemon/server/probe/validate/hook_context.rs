//! Hook context target validation.

use super::report::{bool_at, int_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

mod graph;

mod receipt;

use receipt::latest_receipt;

pub(super) fn hook_context_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("hook:")
        .or_else(|| target.strip_prefix("hook/"))
        .or_else(|| target.strip_prefix("hook_context:"))
        .or_else(|| target.strip_prefix("hook_context/"))
        .and_then(|rest| rest.split('/').next())
        .and_then(|id| id.split('@').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn hook_context_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    session_id: &str,
) -> Value {
    let session_channel = session_channel_evidence(state, session_id);
    let graph = graph::evidence(state, session_id, &session_channel);
    let explanation = match state.with_store(|s| {
        crate::explain::explain(
            s,
            &crate::explain::Handle::Hook {
                id: session_id.to_string(),
                at: hook_at(target),
            },
        )
    }) {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "session_id": session_id,
                "supported": true,
                "found": bool_at(&graph, "graph_found"),
                "graph": graph,
                "error": e.to_string(),
                "summary": "hook context evidence could not read persisted receipts",
                "reason": e.to_string(),
            });
        }
    };
    let receipt = latest_receipt(&explanation);
    let graph_found = bool_at(&graph, "graph_found");
    let receipt_found = receipt.is_some();
    let receipt_revision = receipt
        .as_ref()
        .and_then(|v| v.get("revision"))
        .and_then(Value::as_i64);
    let graph_revision = graph.get("revision").and_then(Value::as_i64);
    let revision_matches = graph_found
        && receipt_found
        && graph_revision.is_some()
        && graph_revision == receipt_revision;
    let rendered_unconfirmed_channel = bool_at(&graph, "rendered_unconfirmed_channel");
    let rendered_legacy_agents_roster = bool_at(&graph, "rendered_legacy_agents_roster");
    let rendered_member_roster = bool_at(&graph, "rendered_member_roster");
    let member_roster_corroborated = !rendered_member_roster
        || (bool_at(&session_channel, "confirmed")
            && bool_at(&session_channel, "membership_snapshot")
            && int_at(&session_channel, "member_count") >= int_at(&graph, "member_rows"));
    let ok = graph_found
        && receipt_found
        && revision_matches
        && !rendered_unconfirmed_channel
        && !rendered_legacy_agents_roster
        && member_roster_corroborated;

    json!({
        "target": target,
        "session_id": session_id,
        "supported": true,
        "found": graph_found || receipt_found,
        "graph_found": graph_found,
        "receipt_found": receipt_found,
        "revision_matches_receipt": revision_matches,
        "member_roster_corroborated": member_roster_corroborated,
        "ok": ok,
        "graph": graph,
        "session_channel": session_channel,
        "receipt": receipt,
        "summary": summary(
            session_id,
            graph_found,
            receipt_found,
            revision_matches,
            rendered_unconfirmed_channel,
            rendered_legacy_agents_roster,
            member_roster_corroborated,
        ),
        "reason": reason(
            graph_found,
            receipt_found,
            revision_matches,
            rendered_unconfirmed_channel,
            rendered_legacy_agents_roster,
            member_roster_corroborated,
        ),
    })
}

pub(super) fn push_hook_context_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty() {
        "failed"
    } else if bool_at(evidence, "ok") {
        "passed"
    } else if !bool_at(evidence, "found") {
        "not_proven"
    } else {
        "failed"
    };
    checks.push(json!({
        "name": "hook_context_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn session_channel_evidence(state: &Arc<DaemonState>, session_id: &str) -> Value {
    match state.with_store(|s| {
        let session = s.get_session(session_id)?;
        let channel = match &session {
            Some(session) => s.get_channel(&session.channel_h)?,
            None => None,
        };
        let members = match &session {
            Some(session) => s.list_channel_members(&session.channel_h)?,
            None => Vec::new(),
        };
        let membership_snapshot = match &session {
            Some(session) => s.has_channel_membership_snapshot(&session.channel_h)?,
            None => false,
        };
        let channel_ref = session
            .as_ref()
            .map(|session| crate::channel_ref::full_channel_ref(s, &session.channel_h))
            .unwrap_or_default();
        Ok::<_, anyhow::Error>((session, channel, members, membership_snapshot, channel_ref))
    }) {
        Ok((Some(session), channel, members, membership_snapshot, channel_ref)) => json!({
            "session_found": true,
            "channel_h": session.channel_h,
            "channel_ref": channel_ref,
            "confirmed": channel.is_some(),
            "channel_name": channel
                .as_ref()
                .map(|c| c.human_name().unwrap_or(&c.name))
                .unwrap_or(""),
            "membership_snapshot": membership_snapshot,
            "member_count": members.len(),
            "admin_count": members.iter().filter(|member| member.role == "admin").count(),
        }),
        Ok((None, _, _, _, _)) => json!({
            "session_found": false,
            "channel_h": "",
            "channel_ref": "",
            "confirmed": false,
            "membership_snapshot": false,
            "member_count": 0,
            "admin_count": 0,
        }),
        Err(e) => json!({
            "session_found": false,
            "channel_h": "",
            "channel_ref": "",
            "confirmed": false,
            "membership_snapshot": false,
            "member_count": 0,
            "admin_count": 0,
            "error": e.to_string(),
        }),
    }
}

fn hook_at(target: &str) -> Option<i64> {
    target
        .strip_prefix("hook:")
        .and_then(|rest| rest.split_once('@').map(|(_, at)| at))
        .and_then(|at| at.parse().ok())
}

fn summary(
    session_id: &str,
    graph_found: bool,
    receipt_found: bool,
    revision_matches: bool,
    rendered_unconfirmed_channel: bool,
    rendered_legacy_agents_roster: bool,
    member_roster_corroborated: bool,
) -> String {
    if rendered_unconfirmed_channel {
        format!("hook context `{session_id}` rendered an unconfirmed channel as active")
    } else if rendered_legacy_agents_roster {
        format!("hook context `{session_id}` rendered legacy local config as an agent roster")
    } else if !member_roster_corroborated {
        format!("hook context `{session_id}` rendered an uncorroborated member roster")
    } else if revision_matches {
        format!("hook context `{session_id}` live graph has a matching receipt")
    } else if graph_found && receipt_found {
        format!("hook context `{session_id}` live graph revision does not match latest receipt")
    } else if graph_found {
        format!("hook context `{session_id}` has a live graph but no persisted receipt")
    } else if receipt_found {
        format!("hook context `{session_id}` has a historical receipt but no live graph")
    } else {
        format!("hook context `{session_id}` has no live graph or persisted receipt")
    }
}

fn reason(
    graph_found: bool,
    receipt_found: bool,
    revision_matches: bool,
    rendered_unconfirmed_channel: bool,
    rendered_legacy_agents_roster: bool,
    member_roster_corroborated: bool,
) -> &'static str {
    if rendered_unconfirmed_channel {
        "hook context must render missing/unverified channels as degraded warnings, not normal channel blocks"
    } else if rendered_legacy_agents_roster {
        "configured local agents must render as available-agents; active channel roster must come from confirmed members/presence"
    } else if !member_roster_corroborated {
        "hook context rendered members, but the session channel does not have a hydrated relay membership snapshot"
    } else if revision_matches {
        ""
    } else if graph_found && receipt_found {
        "live hook_context graph revision does not match the latest persisted hook receipt"
    } else if graph_found {
        "live hook_context graph has no persisted receipt, so the injected context is not explainable"
    } else if receipt_found {
        "persisted hook receipt is historical; no live hook_context graph is materialized for this session"
    } else {
        "no hook_context graph or receipt was found for this session"
    }
}
