//! Joined-channel evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "joined-channel evidence");
    let _ = writeln!(
        out,
        "  - session={} active={} alive={} joined={} requested={:?}",
        str_at(evidence, "session_id"),
        str_at(evidence, "active_channel_h"),
        bool_at(evidence, "session_alive"),
        int_at(evidence, "joined_count"),
        str_at(evidence, "channel_h")
    );
    render_rows(out, evidence.get("rows"));
    if int_at(evidence, "missing_subscription_count") > 0 {
        let _ = writeln!(
            out,
            "  - missing_subscription_count={}",
            int_at(evidence, "missing_subscription_count")
        );
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn render_rows(out: &mut String, value: Option<&Value>) {
    let rows = value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    for row in rows.iter().take(6) {
        let _ = writeln!(
            out,
            "  - {} joined_at={} channel_found={} sub_h={} sub_d={}",
            str_at(row, "channel_h"),
            int_at(row, "joined_at"),
            bool_at(row, "channel_found"),
            bool_at(row, "sub_h_owned"),
            bool_at(row, "sub_d_owned")
        );
    }
    if rows.len() > 6 {
        let _ = writeln!(out, "  - ... {} more channel(s)", rows.len() - 6);
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
