//! Quarantine evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "quarantine evidence");
    let _ = writeln!(
        out,
        "  - event_prefix={} rows={} materialized={} message={} relay_event={}",
        str_at(evidence, "event_prefix"),
        int_at(evidence, "row_count"),
        bool_at(evidence, "materialized"),
        bool_at(evidence, "message_found"),
        bool_at(evidence, "relay_event_found")
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
            "  - {} kind={} channel={} author={} quarantined_at={} reason={}",
            str_at(row, "id"),
            int_at(row, "kind"),
            str_at(row, "channel_h"),
            str_at(row, "pubkey"),
            int_at(row, "quarantined_at"),
            str_at(row, "reason")
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
