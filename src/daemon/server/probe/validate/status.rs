//! Status target validation for live publish state.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

pub(super) fn status_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("status:")
        .or_else(|| target.strip_prefix("status/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn status_evidence(state: &Arc<DaemonState>, target: &str, session_id: &str) -> Value {
    let graph_row = state
        .status
        .lock()
        .expect("status mutex poisoned")
        .state_rows()
        .into_iter()
        .find(|row| row.session == session_id);
    let durable = state.with_store(|s| {
        let session = s.get_session(session_id)?;
        let mut relay_rows = s
            .list_status_sessions(None, None)?
            .into_iter()
            .filter(|row| row.session_id == session_id)
            .collect::<Vec<_>>();
        relay_rows.sort_by(|a, b| {
            b.last_seen
                .cmp(&a.last_seen)
                .then_with(|| a.channel_h.cmp(&b.channel_h))
        });
        Ok::<_, anyhow::Error>((session, relay_rows))
    });
    let (session, relay_rows) = match durable {
        Ok(rows) => rows,
        Err(e) => {
            return json!({
                "target": target,
                "session_id": session_id,
                "supported": true,
                "found": false,
                "graph_found": graph_row.is_some(),
                "relay_status_found": false,
                "error": e.to_string(),
                "summary": "status evidence could not read durable state",
                "reason": e.to_string(),
            });
        }
    };
    let now = crate::util::now_secs();
    let live_rows = relay_rows
        .iter()
        .filter(|row| row.expiration >= now)
        .collect::<Vec<_>>();
    let latest = live_rows.first().copied().or_else(|| relay_rows.first());
    let graph_found = graph_row.is_some();
    let session_row_found = session.is_some();
    let session_alive = session.as_ref().is_some_and(|s| s.alive);
    let relay_status_found = !relay_rows.is_empty();
    let relay_live_count = live_rows.len();
    let graph_channels = graph_row
        .as_ref()
        .map(|row| row.channels.clone())
        .unwrap_or_default();
    let relay_channels = string_set(relay_rows.iter().map(|row| row.channel_h.as_str()));
    let relay_live_channels = string_set(live_rows.iter().map(|row| row.channel_h.as_str()));
    let relay_pubkeys = string_set(relay_rows.iter().map(|row| row.pubkey.as_str()));
    let relay_slugs = string_set(
        relay_rows
            .iter()
            .map(|row| row.slug.as_str())
            .filter(|slug| !slug.is_empty()),
    );
    let channel_h = session.as_ref().map(|s| s.channel_h.as_str()).unwrap_or("");
    let channel_confirmed = !channel_h.is_empty() && graph_channels.iter().any(|h| h == channel_h);
    let busy_matches = session
        .as_ref()
        .map(|s| graph_row.as_ref().is_some_and(|row| row.busy == s.working));
    let title_matches = session
        .as_ref()
        .map(|s| graph_row.as_ref().is_some_and(|row| row.title == s.title));

    json!({
        "target": target,
        "session_id": session_id,
        "supported": true,
        "found": graph_found || session_row_found || relay_status_found,
        "graph_found": graph_found,
        "session_row_found": session_row_found,
        "session_alive": session_alive,
        "relay_status_found": relay_status_found,
        "relay_status_live": relay_live_count > 0,
        "relay_status_count": relay_rows.len(),
        "relay_live_count": relay_live_count,
        "relay_expired_count": relay_rows.len().saturating_sub(relay_live_count),
        "relay_channels": relay_channels,
        "relay_live_channels": relay_live_channels,
        "relay_pubkeys": relay_pubkeys,
        "relay_slugs": relay_slugs,
        "relay_pubkey": latest.map(|r| r.pubkey.as_str()).unwrap_or(""),
        "relay_slug": latest.map(|r| r.slug.as_str()).unwrap_or(""),
        "relay_title": latest.map(|r| r.title.as_str()).unwrap_or(""),
        "relay_activity": latest.map(|r| r.activity.as_str()).unwrap_or(""),
        "relay_busy": latest.map(|r| r.busy).unwrap_or(false),
        "relay_last_seen": latest.map(|r| r.last_seen).unwrap_or(0),
        "relay_expiration": latest.map(|r| r.expiration).unwrap_or(0),
        "relay_now": now,
        "graph_title": graph_row.as_ref().map(|r| r.title.as_str()).unwrap_or(""),
        "graph_activity": graph_row.as_ref().map(|r| r.activity.as_str()).unwrap_or(""),
        "graph_busy": graph_row.as_ref().map(|r| r.busy).unwrap_or(false),
        "graph_channels": graph_channels,
        "channel_h": channel_h,
        "agent_slug": session.as_ref().map(|s| s.agent_slug.as_str()).unwrap_or(""),
        "harness": session.as_ref().map(|s| s.harness.as_str()).unwrap_or(""),
        "local_title": session.as_ref().map(|s| s.title.as_str()).unwrap_or(""),
        "local_activity": session.as_ref().map(|s| s.activity.as_str()).unwrap_or(""),
        "local_working": session.as_ref().map(|s| s.working).unwrap_or(false),
        "last_seen": session.as_ref().map(|s| s.last_seen).unwrap_or(0),
        "last_distill_at": session.as_ref().map(|s| s.last_distill_at).unwrap_or(0),
        "channel_confirmed": channel_confirmed,
        "busy_matches_session": busy_matches,
        "title_matches_session": title_matches,
        "summary": summary(session_id, graph_found, session_row_found, session_alive, relay_rows.len(), relay_live_count, &graph_row),
        "reason": reason(graph_found, session_row_found, session_alive, relay_status_found, relay_live_count),
    })
}

pub(super) fn push_status_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty() {
        "failed"
    } else if bool_at(evidence, "graph_found")
        && bool_at(evidence, "session_row_found")
        && !bool_at(evidence, "session_alive")
    {
        "failed"
    } else if !bool_at(evidence, "graph_found") && bool_at(evidence, "session_alive") {
        "failed"
    } else if bool_at(evidence, "graph_found") {
        "passed"
    } else if bool_at(evidence, "relay_status_live") {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "status_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn summary(
    session_id: &str,
    graph_found: bool,
    session_row_found: bool,
    session_alive: bool,
    relay_count: usize,
    relay_live_count: usize,
    graph_row: &Option<crate::reconcile::status::probe::StatusStateRow>,
) -> String {
    match (
        graph_found,
        session_row_found,
        session_alive,
        relay_count,
        relay_live_count,
        graph_row,
    ) {
        (true, _, _, _, _, Some(row)) => format!(
            "status `{session_id}` is published busy={} channels={}",
            row.busy,
            row.channels.len()
        ),
        (false, true, true, _, _, _) => {
            format!("session `{session_id}` is alive locally but has no live status row")
        }
        (false, _, _, _, live, _) if live > 0 => {
            format!("status `{session_id}` is relay-live in {live} channel(s)")
        }
        (false, true, false, _, _, _) => {
            format!("session `{session_id}` is local but dead with no live status row")
        }
        (false, _, _, count, 0, _) if count > 0 => {
            format!("status `{session_id}` has relay status rows, but none are live")
        }
        _ => format!("status `{session_id}` has no live graph, relay, or local session evidence"),
    }
}

fn reason(
    graph_found: bool,
    session_row_found: bool,
    session_alive: bool,
    relay_status_found: bool,
    relay_live_count: usize,
) -> &'static str {
    match (
        graph_found,
        session_row_found,
        session_alive,
        relay_status_found,
        relay_live_count,
    ) {
        (false, true, true, _, _) => {
            "local session row is alive, but no status command is live for it"
        }
        (true, true, false, _, _) => "status graph is live, but the local session row is dead",
        (true, false, _, _, _) => {
            "status graph is live, but no local session row exists to tie it to a hosted process"
        }
        (false, false, _, true, live) if live > 0 => {
            "relay status is materialized, but no local Trellis status graph exists on this daemon"
        }
        (false, _, _, true, 0) => "relay status rows exist, but all are expired",
        (false, false, _, false, _) => {
            "no status command, relay status row, or local session row was found for this session"
        }
        _ => "",
    }
}

fn string_set<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    values
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
