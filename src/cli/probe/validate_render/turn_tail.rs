//! Turn lifecycle evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "turn evidence");
    if !bool_at(evidence, "found") {
        let _ = writeln!(out, "  - {}: not found", str_at(evidence, "session_id"));
    } else {
        let _ = writeln!(
            out,
            "  - {}: graph={} session={}",
            str_at(evidence, "session_id"),
            present(evidence, "graph_found"),
            session_state(evidence)
        );
        let _ = writeln!(
            out,
            "  - graph working={} started={} transcript={:?}",
            bool_at(evidence, "graph_working"),
            int_at(evidence, "graph_turn_started_at"),
            str_at(evidence, "graph_transcript_ref")
        );
        if bool_at(evidence, "session_row_found") {
            let _ = writeln!(
                out,
                "  - local working={} started={} transcript={:?}",
                bool_at(evidence, "local_working"),
                int_at(evidence, "local_turn_started_at"),
                str_at(evidence, "local_transcript_path")
            );
        }
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn present(evidence: &Value, key: &str) -> &'static str {
    if bool_at(evidence, key) {
        "present"
    } else {
        "missing"
    }
}

fn session_state(evidence: &Value) -> &'static str {
    if !bool_at(evidence, "session_row_found") {
        "missing"
    } else if bool_at(evidence, "session_alive") {
        "alive"
    } else {
        "dead"
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
