//! Event-level validation evidence for `probe validate`.

use super::report::{bool_at, int_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

const EVENT_LIMITATION: &str = "event validation can prove local materialization; Trellis explanation is available only when a receipt records this event as an artifact";

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
            .map(|event| relay_context(store, event))
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
            .or_else(|| outbox.iter().find_map(|r| event_json_id(&r.event_json)))
            .or_else(|| receipts.iter().find_map(|r| r.artifact_ref.clone()))
            .or_else(|| quarantine.first().map(|r| r.id.clone()))
            .unwrap_or_else(|| requested_id.to_string());
        let outbox_rows = outbox.iter().take(5).map(outbox_json).collect::<Vec<_>>();
        let graph_outbox_rows = graph_outbox
            .iter()
            .take(5)
            .map(graph_outbox_json)
            .collect::<Vec<_>>();
        let quarantine_rows = quarantine
            .iter()
            .take(5)
            .map(quarantine_json)
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
            "relay_validation_reason": relay_event.as_ref().map(|event| relay_validation_reason(event, &relay_context)).unwrap_or(""),
            "summary": summary(requested_id, &resolved_id, receipts.len(), message.as_ref(), relay_event.as_ref(), !quarantine.is_empty(), outbox_published, outbox_pending),
            "reason": reason(found, receipts.len(), message.as_ref(), relay_event.is_some(), !quarantine.is_empty(), outbox_failed, outbox_pending, outbox_published),
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
    } else if !bool_at(evidence, "found") {
        "not_proven"
    } else if provisional_message(evidence)
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
        push_relay_limitations(limitations, evidence);
    }
}

#[derive(Default)]
struct RelayContext {
    tags_valid: bool,
    tag_count: i64,
    channel_found: bool,
    channel_name: String,
    author_profile_found: bool,
    author_slug: String,
    membership_snapshot: bool,
    author_role: String,
}

fn relay_context(
    store: &crate::state::Store,
    event: &crate::state::RelayEvent,
) -> anyhow::Result<RelayContext> {
    let tag_count = serde_json::from_str::<Value>(&event.tags_json)
        .ok()
        .and_then(|value| value.as_array().map(|tags| tags.len() as i64));
    let channel = if event.channel_h.is_empty() {
        None
    } else {
        store.get_channel(&event.channel_h)?
    };
    let profile = store.get_profile(&event.pubkey)?;
    let membership_snapshot = if event.channel_h.is_empty() {
        false
    } else {
        store.has_channel_membership_snapshot(&event.channel_h)?
    };
    let author_role = if event.channel_h.is_empty() {
        String::new()
    } else {
        store
            .list_channel_members(&event.channel_h)?
            .into_iter()
            .find(|member| member.pubkey == event.pubkey)
            .map(|member| member.role)
            .unwrap_or_default()
    };
    Ok(RelayContext {
        tags_valid: tag_count.is_some(),
        tag_count: tag_count.unwrap_or(-1),
        channel_found: channel.is_some(),
        channel_name: channel
            .as_ref()
            .map(|row| row.name.clone())
            .unwrap_or_default(),
        author_profile_found: profile.is_some(),
        author_slug: profile
            .as_ref()
            .map(|row| row.slug.clone())
            .unwrap_or_default(),
        membership_snapshot,
        author_role,
    })
}

fn relay_validation_reason(
    event: &crate::state::RelayEvent,
    context: &RelayContext,
) -> &'static str {
    if !context.tags_valid {
        return "relay event tags_json is not valid JSON array data";
    }
    if event.kind == crate::fabric::nip29::wire::KIND_CHAT as u32
        && !event.channel_h.is_empty()
        && context.membership_snapshot
        && context.author_role.is_empty()
    {
        return "hydrated channel membership snapshot does not include relay event author";
    }
    ""
}

fn push_relay_limitations(limitations: &mut Vec<String>, evidence: &Value) {
    let relay_channel = str_at(evidence, "relay_channel_h");
    if !relay_channel.is_empty() && !bool_at(evidence, "relay_channel_found") {
        limitations.push("relay event channel metadata is not materialized".to_string());
    }
    if !bool_at(evidence, "relay_author_profile_found") {
        limitations.push("relay event author profile is not materialized".to_string());
    }
    if int_at(evidence, "relay_kind") == crate::fabric::nip29::wire::KIND_CHAT as i64
        && !relay_channel.is_empty()
        && !bool_at(evidence, "relay_membership_snapshot")
    {
        limitations.push(
            "relay event author membership cannot be proven until channel roster snapshots hydrate"
                .to_string(),
        );
    }
}

fn summary(
    requested_id: &str,
    resolved_id: &str,
    receipt_count: usize,
    message: Option<&crate::state::Message>,
    relay_event: Option<&crate::state::RelayEvent>,
    quarantine_found: bool,
    outbox_published: bool,
    outbox_pending: bool,
) -> String {
    if quarantine_found {
        return format!("event `{resolved_id}` is quarantined before normal materialization");
    }
    if receipt_count > 0 {
        if outbox_published {
            return format!(
                "event `{resolved_id}` has {receipt_count} Trellis receipt(s) and published outbox evidence"
            );
        }
        return format!("event `{resolved_id}` has {receipt_count} Trellis receipt(s)");
    }
    if let Some(message) = message {
        return format!(
            "event `{}` is a chat message with sync_state `{}` in channel `{}`",
            message.message_id, message.sync_state, message.channel_h
        );
    }
    if let Some(event) = relay_event {
        return format!(
            "event `{}` is cached as relay kind {} in channel `{}`",
            event.id, event.kind, event.channel_h
        );
    }
    if outbox_published {
        return format!("event `{resolved_id}` is published in the outbox ledger");
    }
    if outbox_pending {
        return format!("event `{resolved_id}` is pending in the outbox ledger");
    }
    format!("event `{requested_id}` is not locally materialized")
}

fn reason(
    found: bool,
    receipt_count: usize,
    message: Option<&crate::state::Message>,
    relay_event_found: bool,
    quarantine_found: bool,
    outbox_failed: bool,
    outbox_pending: bool,
    outbox_published: bool,
) -> &'static str {
    if !found {
        return "no Trellis receipt, outbox row, canonical message row, or relay event row matched this event id prefix";
    }
    if quarantine_found {
        return "relay event is quarantined and has not been admitted to canonical event/message state";
    }
    if outbox_failed {
        return "outbox row records a failed relay publish outcome";
    }
    if let Some(message) = message {
        if message.error.as_deref().is_some_and(|s| !s.is_empty())
            || is_failed_state(&message.sync_state)
        {
            return "canonical message row records a failed/rejected sync state";
        }
        if is_provisional_state(&message.sync_state) || message.native_event_id.is_none() {
            return "canonical message row exists, but relay/native event materialization is not proven";
        }
    }
    if outbox_pending && !outbox_published {
        return "outbox row exists, but relay acceptance is still pending";
    }
    if receipt_count > 0 {
        if outbox_published {
            return "Trellis receipts explain this event artifact and outbox evidence proves relay publish completion";
        }
        return "Trellis receipts explain this event artifact";
    }
    if outbox_published {
        return "outbox evidence proves relay publish completion, but no Trellis receipt explains this event";
    }
    if message.is_some() {
        return "canonical message row proves local chat materialization, but no Trellis receipt explains this event";
    }
    if relay_event_found {
        return "raw relay cache proves local materialization, but no Trellis receipt explains this event";
    }
    EVENT_LIMITATION
}

fn outbox_json(row: &crate::state::OutboxRow) -> Value {
    json!({
        "local_id": row.local_id,
        "state": row.state,
        "retries": row.retries,
        "last_error": row.last_error,
        "enqueued_at": row.enqueued_at,
        "event_json_id": event_json_id(&row.event_json).unwrap_or_default(),
    })
}

fn graph_outbox_json(row: &crate::reconcile::outbox::OutboxStateRow) -> Value {
    json!({
        "local_id": row.local_id,
        "event_id": row.event_id,
        "state": row.state,
        "retries": row.retries,
        "last_error": row.last_error,
        "source_ref": row.source_ref,
    })
}

fn quarantine_json(row: &crate::state::QuarantinedEvent) -> Value {
    json!({
        "id": row.id,
        "kind": row.kind,
        "pubkey": row.pubkey,
        "created_at": row.created_at,
        "channel_h": row.channel_h,
        "reason": row.reason,
        "quarantined_at": row.quarantined_at,
        "event_json_id": event_json_id(&row.event_json).unwrap_or_default(),
    })
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

fn event_json_id(event_json: &str) -> Option<String> {
    serde_json::from_str::<Value>(event_json)
        .ok()
        .and_then(|v| v.get("id").and_then(Value::as_str).map(str::to_string))
}
