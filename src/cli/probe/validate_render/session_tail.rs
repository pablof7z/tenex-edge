//! Session evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "session evidence");
    if !bool_at(evidence, "found") {
        let _ = writeln!(out, "  - {}: not found", str_at(evidence, "session_id"));
    } else {
        let _ = writeln!(
            out,
            "  - {}: {} channel={} agent={} harness={}",
            str_at(evidence, "session_id"),
            if bool_at(evidence, "alive") {
                "alive"
            } else {
                "dead"
            },
            str_at(evidence, "channel_h"),
            str_at(evidence, "agent_slug"),
            str_at(evidence, "harness")
        );
        let _ = writeln!(
            out,
            "  - status={} watch={} sub_h={} sub_d={} working={} last_seen={}",
            yn(evidence, "status_found"),
            yn(evidence, "watch_found"),
            yn(evidence, "sub_h_owned"),
            yn(evidence, "sub_d_owned"),
            bool_at(evidence, "working"),
            int_at(evidence, "last_seen")
        );
        if !missing(evidence).is_empty() {
            let _ = writeln!(out, "  - missing={}", missing(evidence));
        }
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
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
