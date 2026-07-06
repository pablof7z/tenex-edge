//! Inbox evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "inbox evidence");
    let _ = writeln!(
        out,
        "  - event_prefix={} target={:?} rows={} events={} pending={} processing={} delivered={} failed={}",
        str_at(evidence, "event_prefix"),
        str_at(evidence, "target_session"),
        int_at(evidence, "row_count"),
        int_at(evidence, "event_count"),
        int_at(evidence, "pending_count"),
        int_at(evidence, "processing_count"),
        int_at(evidence, "delivered_count"),
        int_at(evidence, "failed_count")
    );
    render_rows(out, evidence.get("rows"));
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn render_rows(out: &mut String, value: Option<&Value>) {
    let rows = value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    for row in rows.iter().take(5) {
        let _ = writeln!(
            out,
            "  - {} -> {} ({}) state={} channel={} session_alive={} body_len={}",
            str_at(row, "event_id"),
            str_at(row, "target_session"),
            str_at(row, "target_kind"),
            str_at(row, "state"),
            str_at(row, "channel_h"),
            bool_at(row, "session_alive"),
            int_at(row, "body_len")
        );
    }
    if rows.len() > 5 {
        let _ = writeln!(out, "  - ... {} more row(s)", rows.len() - 5);
    }
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
