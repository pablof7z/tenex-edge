//! Session alias evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "alias evidence");
    let _ = writeln!(
        out,
        "  - kind={} harness={:?} external_id={} rows={} live={} session={}",
        str_at(evidence, "alias_kind"),
        str_at(evidence, "harness"),
        str_at(evidence, "external_id"),
        int_at(evidence, "row_count"),
        bool_at(evidence, "resolved_live"),
        str_at(evidence, "resolved_session_id")
    );
    if bool_at(evidence, "session_found") {
        let _ = writeln!(
            out,
            "  - session alive={} channel={} agent={}",
            bool_at(evidence, "session_alive"),
            str_at(evidence, "channel_h"),
            str_at(evidence, "agent_slug")
        );
        let _ = writeln!(
            out,
            "  - status={} watch={} sub_h={} sub_d={}",
            yn(evidence, "status_found"),
            yn(evidence, "watch_found"),
            yn(evidence, "sub_h_owned"),
            yn(evidence, "sub_d_owned")
        );
    }
    render_rows(out, evidence.get("rows"));
    if !missing(evidence).is_empty() {
        let _ = writeln!(out, "  - missing={}", missing(evidence));
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
    for row in rows.iter().take(3) {
        let _ = writeln!(
            out,
            "  - {}:{}:{} -> {} alive={} created_at={}",
            str_at(row, "harness"),
            str_at(row, "external_id_kind"),
            str_at(row, "external_id"),
            str_at(row, "session_id"),
            bool_at(row, "session_alive"),
            int_at(row, "created_at")
        );
    }
    if rows.len() > 3 {
        let _ = writeln!(out, "  - ... {} more alias row(s)", rows.len() - 3);
    }
}

fn missing(evidence: &Value) -> String {
    evidence
        .get("missing")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default()
}

fn yn(v: &Value, key: &str) -> &'static str {
    if bool_at(v, key) {
        "yes"
    } else {
        "no"
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
