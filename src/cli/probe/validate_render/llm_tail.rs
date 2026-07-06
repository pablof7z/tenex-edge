//! LLM evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "llm evidence");
    if !bool_at(evidence, "call_found") {
        let _ = writeln!(out, "  - llm:{} not found", int_at(evidence, "llm_id"));
    } else {
        let session = if bool_at(evidence, "session_row_found") {
            if bool_at(evidence, "session_alive") {
                "alive"
            } else {
                "dead"
            }
        } else {
            "missing"
        };
        let _ = writeln!(
            out,
            "  - llm:{} {} / {} session={} ({session})",
            int_at(evidence, "llm_id"),
            str_at(evidence, "provider"),
            str_at(evidence, "model"),
            str_at(evidence, "session_id")
        );
        let _ = writeln!(
            out,
            "  - window={} receipts={} title={:?} activity={:?}",
            str_at(evidence, "window_hash"),
            int_at(evidence, "receipt_count"),
            str_at(evidence, "parsed_title"),
            str_at(evidence, "parsed_activity")
        );
        let _ = writeln!(
            out,
            "  - bytes system={} transcript={} response={}",
            int_at(evidence, "system_prompt_bytes"),
            int_at(evidence, "transcript_slice_bytes"),
            int_at(evidence, "raw_response_bytes")
        );
        render_strings(out, "artifacts", evidence.get("receipt_artifacts"));
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn render_strings(out: &mut String, label: &str, value: Option<&Value>) {
    let items = value
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    if !items.is_empty() {
        let _ = writeln!(out, "  - {label}={}", items.join(","));
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
