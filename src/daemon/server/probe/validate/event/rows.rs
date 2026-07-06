use serde_json::{json, Value};

pub(super) fn event_json_id(event_json: &str) -> Option<String> {
    serde_json::from_str::<Value>(event_json)
        .ok()
        .and_then(|v| v.get("id").and_then(Value::as_str).map(str::to_string))
}

pub(super) fn outbox_json(row: &crate::state::OutboxRow) -> Value {
    json!({
        "local_id": row.local_id,
        "state": row.state,
        "retries": row.retries,
        "last_error": row.last_error,
        "enqueued_at": row.enqueued_at,
        "event_json_id": event_json_id(&row.event_json).unwrap_or_default(),
    })
}

pub(super) fn graph_outbox_json(row: &crate::reconcile::outbox::OutboxStateRow) -> Value {
    json!({
        "local_id": row.local_id,
        "event_id": row.event_id,
        "state": row.state,
        "retries": row.retries,
        "last_error": row.last_error,
        "source_ref": row.source_ref,
    })
}

pub(super) fn quarantine_json(row: &crate::state::QuarantinedEvent) -> Value {
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
