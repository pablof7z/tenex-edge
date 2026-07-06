//! Hook context target validation.

use super::report::{bool_at, int_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

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
    let graph = graph_evidence(state, session_id, &session_channel);
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

fn graph_evidence(state: &Arc<DaemonState>, session_id: &str, session_channel: &Value) -> Value {
    let graphs = state
        .hook_contexts
        .lock()
        .expect("hook-context mutex poisoned");
    let Some(graph) = graphs.get(session_id) else {
        return json!({
            "graph_found": false,
            "resource_key": format!("hook/{session_id}/view"),
        });
    };
    let text = graph.current_text();
    let channel_h = str_at(session_channel, "channel_h");
    let channel_confirmed = bool_at(session_channel, "confirmed");
    let rendered_unconfirmed_channel = text
        .as_ref()
        .is_some_and(|text| !channel_confirmed && renders_channel_block(text, channel_h));
    let missing_channel_warning_rendered = text
        .as_ref()
        .is_some_and(|text| renders_missing_channel_warning(text, channel_h));
    let rendered_local_agents = text.as_ref().is_some_and(|text| renders_local_agents(text));
    let rendered_member_roster = text
        .as_ref()
        .is_some_and(|text| renders_member_roster(text));
    let rendered_legacy_agents_roster = text
        .as_ref()
        .is_some_and(|text| renders_legacy_agents_roster(text));
    json!({
        "graph_found": true,
        "resource_key": graph
            .view_label()
            .unwrap_or_else(|| format!("hook/{session_id}/view")),
        "revision": graph.revision(),
        "nodes": graph.graph_node_count(),
        "render_count": graph.render_count(),
        "emitted": text.is_some(),
        "text_bytes": text.as_ref().map(String::len).unwrap_or(0),
        "rendered_unconfirmed_channel": rendered_unconfirmed_channel,
        "missing_channel_warning_rendered": missing_channel_warning_rendered,
        "rendered_local_agents": rendered_local_agents,
        "rendered_member_roster": rendered_member_roster,
        "rendered_legacy_agents_roster": rendered_legacy_agents_roster,
        "local_agent_rows": text.as_ref().map(|text| count_marker(text, "<agent ref=\"@")).unwrap_or(0),
        "member_rows": text.as_ref().map(|text| count_marker(text, "<member ref=\"@")).unwrap_or(0),
        "input_labels": graph.input_labels(),
        "why_input_causes": graph.why_view_input_causes(),
    })
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
        Ok::<_, anyhow::Error>((session, channel, members, membership_snapshot))
    }) {
        Ok((Some(session), channel, members, membership_snapshot)) => json!({
            "session_found": true,
            "channel_h": session.channel_h,
            "confirmed": channel.is_some(),
            "channel_name": channel
                .as_ref()
                .map(|c| c.human_name().unwrap_or(&c.name))
                .unwrap_or(""),
            "membership_snapshot": membership_snapshot,
            "member_count": members.len(),
            "admin_count": members.iter().filter(|member| member.role == "admin").count(),
        }),
        Ok((None, _, _, _)) => json!({
            "session_found": false,
            "channel_h": "",
            "confirmed": false,
            "membership_snapshot": false,
            "member_count": 0,
            "admin_count": 0,
        }),
        Err(e) => json!({
            "session_found": false,
            "channel_h": "",
            "confirmed": false,
            "membership_snapshot": false,
            "member_count": 0,
            "admin_count": 0,
            "error": e.to_string(),
        }),
    }
}

fn latest_receipt(explanation: &Value) -> Option<Value> {
    let receipt = explanation
        .get("receipts")
        .and_then(Value::as_array)?
        .first()?;
    let changed = serde_json::from_str::<Value>(str_at(receipt, "changed_summary")).ok();
    Some(json!({
        "id": receipt.get("id").and_then(Value::as_i64),
        "transaction_id": receipt.get("transaction_id").and_then(Value::as_i64),
        "revision": receipt.get("revision").and_then(Value::as_i64),
        "artifact_ref": receipt.get("artifact_ref").and_then(Value::as_str),
        "created_at": receipt.get("created_at").and_then(Value::as_i64),
        "kind": changed.as_ref().and_then(|v| v.get("kind")).and_then(Value::as_str),
        "shape": changed.as_ref().and_then(|v| v.get("shape")).and_then(Value::as_str),
        "frame": changed.as_ref().and_then(|v| v.get("frame")).and_then(Value::as_str),
        "emitted": changed.as_ref()
            .and_then(|v| v.pointer("/output/emitted"))
            .and_then(Value::as_bool),
        "bytes": changed.as_ref()
            .and_then(|v| v.pointer("/output/bytes"))
            .and_then(Value::as_u64),
        "input_causes": changed.as_ref()
            .and_then(|v| v.get("input_causes"))
            .cloned()
            .unwrap_or_else(|| json!([])),
    }))
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
        format!("hook context `{session_id}` rendered local config as an agent roster")
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
        "configured local agents must render as local-agents; active channel roster must come from confirmed members/presence"
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

fn renders_channel_block(text: &str, channel_h: &str) -> bool {
    !channel_h.is_empty() && text.contains(&format!("<channel name=\"#{channel_h}\""))
}

fn renders_missing_channel_warning(text: &str, channel_h: &str) -> bool {
    !channel_h.is_empty()
        && text.contains(&format!("Fabric channel \"{channel_h}\" is unavailable"))
}

fn renders_local_agents(text: &str) -> bool {
    text.contains("<local-agents>")
}

fn renders_member_roster(text: &str) -> bool {
    text.contains("<members>")
}

fn renders_legacy_agents_roster(text: &str) -> bool {
    text.contains("<agents>")
}

fn count_marker(text: &str, marker: &str) -> usize {
    text.match_indices(marker).count()
}
