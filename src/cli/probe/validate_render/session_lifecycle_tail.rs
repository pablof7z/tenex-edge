//! Session-start and session-watch evidence renderers.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render_session_start(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "session start evidence");
    if !bool_at(evidence, "found") {
        let _ = writeln!(out, "  - {}: not found", str_at(evidence, "session_id"));
    } else {
        let _ = writeln!(
            out,
            "  - {}: {} channel={} reassert={}",
            str_at(evidence, "session_id"),
            str_at(evidence, "action"),
            str_at(evidence, "channel_h"),
            bool_at(evidence, "reassert")
        );
        let mut intents = Vec::new();
        if bool_at(evidence, "has_channel_ready_intent") {
            intents.push("channel_ready");
        }
        if bool_at(evidence, "has_spawn_intent") {
            intents.push("spawn");
        }
        if bool_at(evidence, "ensure_subscription") {
            intents.push("subscription");
        }
        if bool_at(evidence, "replay_chat") {
            intents.push("chat_replay");
        }
        if !intents.is_empty() {
            let _ = writeln!(out, "  - planned host effects: {}", intents.join(", "));
        }
        if !str_at(evidence, "failure_stage").is_empty()
            || !str_at(evidence, "failure_error").is_empty()
        {
            let _ = writeln!(
                out,
                "  - failed at {}: {}",
                str_at(evidence, "failure_stage"),
                str_at(evidence, "failure_error")
            );
        }
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

pub(super) fn render_session_watch(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "session watch evidence");
    let graph = if bool_at(evidence, "graph_open") {
        "open"
    } else {
        "closed"
    };
    let row = if bool_at(evidence, "session_row_found") {
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
        "  - {}: graph={} session_row={}",
        str_at(evidence, "session_id"),
        graph,
        row
    );
    if !str_at(evidence, "channel_h").is_empty() || !str_at(evidence, "agent_slug").is_empty() {
        let _ = writeln!(
            out,
            "  - channel={} agent={} last_seen={}",
            str_at(evidence, "channel_h"),
            str_at(evidence, "agent_slug"),
            int_at(evidence, "last_seen")
        );
    }
    if let Some(pid) = evidence.get("child_pid").and_then(Value::as_i64) {
        let alive = evidence
            .get("process_alive")
            .and_then(Value::as_bool)
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let _ = writeln!(out, "  - pid={pid} process_alive={alive}");
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
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
