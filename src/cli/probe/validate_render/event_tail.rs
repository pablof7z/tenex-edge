//! Event evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "event evidence");
    if !bool_at(evidence, "found") {
        let _ = writeln!(out, "  - {}: not found", str_at(evidence, "requested_id"));
    } else {
        let _ = writeln!(out, "  - {}: materialized", str_at(evidence, "event_id"));
        let receipts = receipt_surfaces(evidence);
        if !receipts.is_empty() {
            let _ = writeln!(
                out,
                "  - receipts={} surfaces={}",
                int_at(evidence, "receipt_count"),
                receipts.join(", ")
            );
        }
        if bool_at(evidence, "message_found") {
            let _ = writeln!(
                out,
                "  - message sync={} channel={} native_event_id={:?}",
                str_at(evidence, "message_sync_state"),
                str_at(evidence, "message_channel_h"),
                str_at(evidence, "native_event_id")
            );
        }
        if bool_at(evidence, "relay_event_found") {
            let _ = writeln!(
                out,
                "  - relay kind={} channel={} author={} content_len={} tags={} valid={}",
                int_at(evidence, "relay_kind"),
                str_at(evidence, "relay_channel_h"),
                str_at(evidence, "relay_author_pubkey"),
                int_at(evidence, "relay_content_len"),
                int_at(evidence, "relay_tag_count"),
                bool_at(evidence, "relay_tags_valid")
            );
            let _ = writeln!(
                out,
                "  - relay channel_found={} name={:?} profile={} slug={:?} membership_snapshot={} role={:?}",
                bool_at(evidence, "relay_channel_found"),
                str_at(evidence, "relay_channel_name"),
                bool_at(evidence, "relay_author_profile_found"),
                str_at(evidence, "relay_author_slug"),
                bool_at(evidence, "relay_membership_snapshot"),
                str_at(evidence, "relay_author_role")
            );
        }
        if bool_at(evidence, "quarantine_found") {
            let _ = writeln!(
                out,
                "  - quarantine rows={} reason={}",
                int_at(evidence, "quarantine_count"),
                first_quarantine_reason(evidence)
            );
        }
        render_outbox(out, evidence);
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn render_outbox(out: &mut String, evidence: &Value) {
    if !bool_at(evidence, "outbox_found") {
        return;
    }
    let _ = writeln!(
        out,
        "  - outbox store={} graph={} published={} pending={} failed={}",
        int_at(evidence, "outbox_store_count"),
        int_at(evidence, "outbox_graph_count"),
        bool_at(evidence, "outbox_published"),
        bool_at(evidence, "outbox_pending"),
        bool_at(evidence, "outbox_failed")
    );
    if let Some(row) = evidence
        .get("outbox_rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .next()
    {
        let _ = writeln!(
            out,
            "  - outbox row id={} state={} retries={} event_json_id={}",
            int_at(row, "local_id"),
            str_at(row, "state"),
            int_at(row, "retries"),
            str_at(row, "event_json_id")
        );
    }
}

fn receipt_surfaces(evidence: &Value) -> Vec<&str> {
    evidence
        .get("receipt_surfaces")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}

fn first_quarantine_reason(evidence: &Value) -> &str {
    evidence
        .get("quarantine_rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .next()
        .and_then(|row| row.get("reason"))
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn str_at<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn bool_at(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn int_at(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}
