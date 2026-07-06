//! Outbox evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "outbox evidence");
    if !bool_at(evidence, "found") {
        let _ = writeln!(
            out,
            "  - outbox/{}: not found",
            int_at(evidence, "local_id")
        );
    } else {
        let _ = writeln!(
            out,
            "  - outbox/{}: graph={} store={}",
            int_at(evidence, "local_id"),
            state_label(evidence, "graph_found", "graph_state"),
            state_label(evidence, "store_row_found", "store_state")
        );
        if !str_at(evidence, "event_id").is_empty() || !str_at(evidence, "event_json_id").is_empty()
        {
            let _ = writeln!(
                out,
                "  - event_id={} durable_event_id={}",
                str_at(evidence, "event_id"),
                str_at(evidence, "event_json_id")
            );
        }
        if !str_at(evidence, "source_ref").is_empty() {
            let _ = writeln!(out, "  - source={}", str_at(evidence, "source_ref"));
        }
        let _ = writeln!(
            out,
            "  - retries graph={} store={} enqueued_at={}",
            int_at(evidence, "graph_retries"),
            int_at(evidence, "store_retries"),
            int_at(evidence, "enqueued_at")
        );
        if !str_at(evidence, "last_error").is_empty() {
            let _ = writeln!(out, "  - error: {}", str_at(evidence, "last_error"));
        }
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn state_label(evidence: &Value, found_key: &str, state_key: &str) -> String {
    if bool_at(evidence, found_key) {
        let state = str_at(evidence, state_key);
        if state.is_empty() {
            "unknown".to_string()
        } else {
            state.to_string()
        }
    } else {
        "missing".to_string()
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
