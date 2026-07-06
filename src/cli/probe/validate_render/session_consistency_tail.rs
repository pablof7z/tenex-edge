//! Session consistency evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let rows = evidence
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if rows.is_empty() && str_at(evidence, "reason").is_empty() {
        return;
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "session consistency evidence");
    let _ = writeln!(
        out,
        "  - sessions={} failed={} live_projections={} uptime={}s",
        int_at(evidence, "session_count"),
        int_at(evidence, "failed_count"),
        int_at(evidence, "live_projection_count"),
        int_at(evidence, "daemon_uptime_secs")
    );
    if bool_at(evidence, "warmup_suspected") {
        let _ = writeln!(out, "  - startup warmup suspected");
    }
    for row in rows.iter().filter(|row| !bool_at(row, "ok")).take(5) {
        let _ = writeln!(
            out,
            "  - {} channel={} missing={}",
            str_at(row, "session_id"),
            str_at(row, "channel_h"),
            missing(row)
        );
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn missing(row: &Value) -> String {
    row.get("missing")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".into())
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
