//! Status evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "status evidence");
    if !bool_at(evidence, "found") {
        let _ = writeln!(
            out,
            "  - status/{}: not found",
            str_at(evidence, "session_id")
        );
    } else {
        let _ = writeln!(
            out,
            "  - status/{}: graph={} session={}",
            str_at(evidence, "session_id"),
            present(evidence, "graph_found"),
            session_state(evidence)
        );
        if bool_at(evidence, "graph_found") {
            let _ = writeln!(
                out,
                "  - published busy={} channels={} title=\"{}\" activity=\"{}\"",
                bool_at(evidence, "graph_busy"),
                channels(evidence),
                str_at(evidence, "graph_title"),
                str_at(evidence, "graph_activity")
            );
        }
        if bool_at(evidence, "session_row_found") {
            let _ = writeln!(
                out,
                "  - local agent={} harness={} channel={} working={} last_seen={}",
                str_at(evidence, "agent_slug"),
                str_at(evidence, "harness"),
                str_at(evidence, "channel_h"),
                bool_at(evidence, "local_working"),
                int_at(evidence, "last_seen")
            );
        }
        if bool_at(evidence, "relay_status_found") {
            let _ = writeln!(
                out,
                "  - relay status live={} rows={} live_rows={} channels={} live_channels={}",
                bool_at(evidence, "relay_status_live"),
                int_at(evidence, "relay_status_count"),
                int_at(evidence, "relay_live_count"),
                array_join(evidence, "relay_channels"),
                array_join(evidence, "relay_live_channels")
            );
            let _ = writeln!(
                out,
                "  - relay published pubkey={} slug={:?} busy={} title=\"{}\" activity=\"{}\" expires={}",
                str_at(evidence, "relay_pubkey"),
                str_at(evidence, "relay_slug"),
                bool_at(evidence, "relay_busy"),
                str_at(evidence, "relay_title"),
                str_at(evidence, "relay_activity"),
                int_at(evidence, "relay_expiration")
            );
        }
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn channels(evidence: &Value) -> String {
    evidence
        .get("graph_channels")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

fn array_join(evidence: &Value, key: &str) -> String {
    evidence
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".to_string())
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
