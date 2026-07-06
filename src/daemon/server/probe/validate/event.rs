//! Event-level validation evidence for `probe validate`.

use super::report::{bool_at, int_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

mod outcome;
mod relay;
mod rows;

pub(super) fn event_target(target: &str) -> Option<&str> {
    target
        .strip_prefix("event:")
        .or_else(|| target.strip_prefix("event/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn event_evidence(state: &Arc<DaemonState>, target: &str, requested_id: &str) -> Value {
    let graph_outbox = {
        let r = state.outbox.lock().expect("outbox mutex poisoned");
        r.state_rows()
            .into_iter()
            .filter(|row| row.event_id.starts_with(requested_id))
            .collect::<Vec<_>>()
    };
    match state.with_store(|store| {
        let receipts = store.receipts_by_artifact_ref_prefix(requested_id)?;
        let message = store.get_message_by_prefix(requested_id)?;
        let relay_event = store.get_event_by_prefix(requested_id)?;
        let relay_context = relay_event
            .as_ref()
            .map(|event| relay::context(store, event))
            .transpose()?
            .unwrap_or_default();
        let outbox = store.outbox_by_event_id_prefix(requested_id)?;
        let quarantine = store.quarantined_events_by_prefix(requested_id)?;
        let receipt_surfaces = receipts
            .iter()
            .map(|r| r.surface.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let resolved_id = message
            .as_ref()
            .map(|m| m.message_id.clone())
            .or_else(|| relay_event.as_ref().map(|e| e.id.clone()))
            .or_else(|| outbox.iter().find_map(|r| rows::event_json_id(&r.event_json)))
            .or_else(|| receipts.iter().find_map(|r| r.artifact_ref.clone()))
            .or_else(|| quarantine.first().map(|r| r.id.clone()))
            .unwrap_or_else(|| requested_id.to_string());
        let outbox_rows = outbox.iter().take(5).map(rows::outbox_json).collect::<Vec<_>>();
        let graph_outbox_rows = graph_outbox
            .iter()
            .take(5)
            .map(rows::graph_outbox_json)
            .collect::<Vec<_>>();
        let quarantine_rows = quarantine
            .iter()
            .take(5)
            .map(rows::quarantine_json)
            .collect::<Vec<_>>();
        let outbox_failed = outbox.iter().any(failed_outbox_row)
            || graph_outbox.iter().any(|row| is_failed_state(&row.state));
        let outbox_pending = outbox.iter().any(|row| pending_state(&row.state))
            || graph_outbox.iter().any(|row| pending_state(&row.state));
        let outbox_published = outbox.iter().any(|row| row.state == "published")
            || graph_outbox.iter().any(|row| row.state == "published");
        let found = !receipts.is_empty()
            || message.is_some()
            || relay_event.is_some()
            || !outbox.is_empty()
            || !graph_outbox.is_empty()
            || !quarantine.is_empty();
        let outcome = outcome::EventOutcome {
            requested_id,
            resolved_id: &resolved_id,
            found,
            receipt_count: receipts.len(),
            message: message.as_ref(),
            relay_event: relay_event.as_ref(),
            quarantine_found: !quarantine.is_empty(),
            outbox_failed,
            outbox_pending,
            outbox_published,
        };
        let summary = outcome::summary(&outcome);
        let reason = outcome::reason(&outcome);

        Ok::<Value, anyhow::Error>(json!({
            "target": target,
            "requested_id": requested_id,
            "event_id": resolved_id,
            "kind": "event",
            "supported": true,
            "found": found,
            "receipt_count": receipts.len(),
            "receipt_surfaces": receipt_surfaces,
            "outbox_store_count": outbox.len(),
            "outbox_graph_count": graph_outbox.len(),
            "outbox_found": !outbox.is_empty() || !graph_outbox.is_empty(),
            "outbox_published": outbox_published,
            "outbox_pending": outbox_pending,
            "outbox_failed": outbox_failed,
            "outbox_rows": outbox_rows,
            "outbox_graph_rows": graph_outbox_rows,
            "quarantine_found": !quarantine.is_empty(),
            "quarantine_count": quarantine.len(),
            "quarantine_rows": quarantine_rows,
            "message_found": message.is_some(),
            "message_channel_h": message.as_ref().map(|m| m.channel_h.as_str()).unwrap_or(""),
            "message_sync_state": message.as_ref().map(|m| m.sync_state.as_str()).unwrap_or(""),
            "message_error": message.as_ref().and_then(|m| m.error.as_deref()).unwrap_or(""),
            "native_event_id": message.as_ref().and_then(|m| m.native_event_id.as_deref()).unwrap_or(""),
            "relay_event_found": relay_event.is_some(),
            "relay_kind": relay_event.as_ref().map(|e| e.kind),
            "relay_channel_h": relay_event.as_ref().map(|e| e.channel_h.as_str()).unwrap_or(""),
            "relay_author_pubkey": relay_event.as_ref().map(|e| e.pubkey.as_str()).unwrap_or(""),
            "relay_content_len": relay_event.as_ref().map(|e| e.content.chars().count()).unwrap_or(0),
            "relay_tags_valid": relay_context.tags_valid,
            "relay_tag_count": relay_context.tag_count,
            "relay_channel_found": relay_context.channel_found,
            "relay_channel_name": relay_context.channel_name,
            "relay_author_profile_found": relay_context.author_profile_found,
            "relay_author_slug": relay_context.author_slug,
            "relay_membership_snapshot": relay_context.membership_snapshot,
            "relay_author_role": relay_context.author_role,
            "relay_author_member_found": !relay_context.author_role.is_empty(),
            "relay_validation_reason": relay_event.as_ref().map(|event| relay::validation_reason(event, &relay_context)).unwrap_or(""),
            "summary": summary,
            "reason": reason,
        }))
    }) {
        Ok(v) => v,
        Err(e) => json!({
            "target": target,
            "requested_id": requested_id,
            "kind": "event",
            "supported": true,
            "found": false,
            "summary": format!("event `{requested_id}` evidence failed: {e}"),
            "reason": e.to_string(),
            "error": e.to_string(),
        }),
    }
}

pub(super) fn push_event_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty()
        || failed_message(evidence)
        || bool_at(evidence, "quarantine_found")
        || bool_at(evidence, "outbox_failed")
        || !str_at(evidence, "relay_validation_reason").is_empty()
    {
        "failed"
    } else if !bool_at(evidence, "found")
        || provisional_message(evidence)
        || (bool_at(evidence, "outbox_pending") && !bool_at(evidence, "outbox_published"))
    {
        "not_proven"
    } else {
        "passed"
    };
    checks.push(json!({
        "name": "event",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" {
        let reason = str_at(evidence, "relay_validation_reason");
        limitations.push(if reason.is_empty() {
            str_at(evidence, "reason").to_string()
        } else {
            reason.to_string()
        });
    } else if int_at(evidence, "receipt_count") == 0 {
        limitations.push(str_at(evidence, "reason").to_string());
    }
    if status == "passed" && status_receipt_without_publish_path(evidence) {
        limitations.push(
            "status receipt has no matching outbox/relay materialization; publish path is not proven"
                .to_string(),
        );
    }
    if status == "passed" && bool_at(evidence, "relay_event_found") {
        relay::push_limitations(limitations, evidence);
    }
}

fn failed_outbox_row(row: &crate::state::OutboxRow) -> bool {
    is_failed_state(&row.state) || row.last_error.as_deref().is_some_and(|s| !s.is_empty())
}

fn failed_message(evidence: &Value) -> bool {
    !str_at(evidence, "message_error").is_empty()
        || is_failed_state(str_at(evidence, "message_sync_state"))
}

fn status_receipt_without_publish_path(evidence: &Value) -> bool {
    let has_status_receipt = evidence
        .get("receipt_surfaces")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|v| v.as_str() == Some("status"));
    has_status_receipt
        && !bool_at(evidence, "outbox_found")
        && !bool_at(evidence, "relay_event_found")
        && str_at(evidence, "native_event_id").is_empty()
}

fn provisional_message(evidence: &Value) -> bool {
    bool_at(evidence, "message_found")
        && (is_provisional_state(str_at(evidence, "message_sync_state"))
            || str_at(evidence, "native_event_id").is_empty())
}

fn is_failed_state(state: &str) -> bool {
    matches!(state, "failed" | "error" | "rejected")
}

fn pending_state(state: &str) -> bool {
    matches!(state, "pending" | "queued" | "sending" | "")
}

fn is_provisional_state(state: &str) -> bool {
    matches!(state, "pending" | "queued" | "sending" | "draft")
}
