//! Quarantine validation for relay events held out of normal materialization.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn quarantine_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("quarantine:")
        .or_else(|| target.strip_prefix("quarantine/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn quarantine_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    event_prefix: &str,
) -> Value {
    let result = state.with_store(|store| {
        let rows = store.quarantined_events_by_prefix(event_prefix)?;
        let (message, message_error) = match store.get_message_by_prefix(event_prefix) {
            Ok(v) => (v, None),
            Err(e) => (None, Some(e.to_string())),
        };
        let (relay_event, relay_event_error) = match store.get_event_by_prefix(event_prefix) {
            Ok(v) => (v, None),
            Err(e) => (None, Some(e.to_string())),
        };
        Ok::<_, anyhow::Error>((rows, message, message_error, relay_event, relay_event_error))
    });
    let (rows, message, message_error, relay_event, relay_event_error) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "event_prefix": event_prefix,
                "kind": "quarantine",
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": "quarantine evidence could not read durable ledger",
                "reason": e.to_string(),
            });
        }
    };

    let ambiguous = rows.len() > 1;
    let materialized = message.is_some() || relay_event.is_some();
    let found = !rows.is_empty();
    let rows_json = rows.iter().take(10).map(row_json).collect::<Vec<_>>();

    json!({
        "target": target,
        "event_prefix": event_prefix,
        "kind": "quarantine",
        "supported": true,
        "found": found,
        "ambiguous": ambiguous,
        "row_count": rows.len(),
        "rows": rows_json,
        "materialized": materialized,
        "message_found": message.is_some(),
        "message_channel_h": message.as_ref().map(|m| m.channel_h.as_str()).unwrap_or(""),
        "message_sync_state": message.as_ref().map(|m| m.sync_state.as_str()).unwrap_or(""),
        "relay_event_found": relay_event.is_some(),
        "relay_kind": relay_event.as_ref().map(|e| e.kind),
        "relay_channel_h": relay_event.as_ref().map(|e| e.channel_h.as_str()).unwrap_or(""),
        "message_lookup_error": message_error.unwrap_or_default(),
        "relay_event_lookup_error": relay_event_error.unwrap_or_default(),
        "ok": !found && materialized,
        "summary": summary(event_prefix, rows.len(), ambiguous, materialized),
        "reason": reason(found, ambiguous, materialized),
    })
}

pub(super) fn push_quarantine_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty()
        || (bool_at(evidence, "found") && !bool_at(evidence, "ambiguous"))
    {
        "failed"
    } else if bool_at(evidence, "ok") {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "quarantine",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn row_json(row: &crate::state::QuarantinedEvent) -> Value {
    json!({
        "id": row.id,
        "kind": row.kind,
        "pubkey": row.pubkey,
        "created_at": row.created_at,
        "channel_h": row.channel_h,
        "reason": row.reason,
        "quarantined_at": row.quarantined_at,
        "event_json_id": event_json_str(&row.event_json, "id"),
        "event_json_kind": event_json_int(&row.event_json, "kind"),
        "content_len": event_json_str(&row.event_json, "content").chars().count(),
    })
}

fn summary(event_prefix: &str, row_count: usize, ambiguous: bool, materialized: bool) -> String {
    if row_count == 0 && materialized {
        return format!("event `{event_prefix}` is not quarantined and is locally materialized");
    }
    if row_count == 0 {
        return format!("event `{event_prefix}` has no quarantine row");
    }
    if ambiguous {
        return format!("quarantine `{event_prefix}` matches multiple events");
    }
    if materialized {
        return format!("event `{event_prefix}` is quarantined but also materialized elsewhere");
    }
    format!("event `{event_prefix}` is quarantined before normal materialization")
}

fn reason(found: bool, ambiguous: bool, materialized: bool) -> &'static str {
    if !found && materialized {
        ""
    } else if !found {
        "no quarantine row or accepted local materialization matched this event id prefix"
    } else if ambiguous {
        "event id prefix matches multiple quarantined events; use a longer id"
    } else if materialized {
        "quarantine row still exists even though the event is materialized"
    } else {
        "relay event is quarantined and has not been admitted to canonical event/message state"
    }
}

fn event_json_str(event_json: &str, key: &str) -> String {
    serde_json::from_str::<Value>(event_json)
        .ok()
        .and_then(|v| v.get(key).and_then(Value::as_str).map(str::to_string))
        .unwrap_or_default()
}

fn event_json_int(event_json: &str, key: &str) -> i64 {
    serde_json::from_str::<Value>(event_json)
        .ok()
        .and_then(|v| v.get(key).and_then(Value::as_i64))
        .unwrap_or(0)
}
