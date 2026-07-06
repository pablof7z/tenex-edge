//! Matched live-state row renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "state evidence");
    let _ = writeln!(
        out,
        "  - {}: {} on {}",
        str_at(evidence, "resource_key"),
        if bool_at(evidence, "found") {
            "found"
        } else {
            "missing"
        },
        str_at(evidence, "surface")
    );
    if let Some(row) = evidence.get("row").filter(|v| v.is_object()) {
        render_row(out, str_at(evidence, "surface"), row);
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

pub(super) fn render_surface_states(out: &mut String, states: &[Value]) {
    let summary_rows = states
        .iter()
        .filter(|state| should_render_summary(state))
        .take(8)
        .collect::<Vec<_>>();
    let problem_rows = states
        .iter()
        .filter_map(|state| {
            let surface = str_at(state, "surface");
            let rows = state.get("rows").and_then(Value::as_array)?;
            Some((surface, rows))
        })
        .flat_map(|(surface, rows)| {
            rows.iter()
                .filter(move |row| aggregate_problem_row(surface, row))
                .map(move |row| (surface, row))
        })
        .take(8)
        .collect::<Vec<_>>();

    if summary_rows.is_empty() && problem_rows.is_empty() {
        return;
    }

    let _ = writeln!(out);
    let _ = writeln!(out, "aggregate state evidence");
    for state in summary_rows {
        render_surface_summary(out, state);
    }
    for (surface, row) in problem_rows {
        match surface {
            "outbox" => render_outbox_problem(out, row),
            _ => {
                let _ = writeln!(out, "  - {} {}", surface, str_at(row, "resource_key"));
            }
        }
    }
}

fn should_render_summary(state: &Value) -> bool {
    let status = str_at(state, "check_status");
    int_at(state, "row_count") > 0 || (!status.is_empty() && status != "passed")
}

fn render_surface_summary(out: &mut String, state: &Value) {
    let sample = state
        .get("sample_targets")
        .and_then(Value::as_array)
        .and_then(|samples| samples.first())
        .and_then(|sample| sample.get("target"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if sample.is_empty() {
        let _ = writeln!(
            out,
            "  - {} {} rows={}",
            str_at(state, "surface"),
            str_at(state, "check_status"),
            int_at(state, "row_count")
        );
    } else {
        let _ = writeln!(
            out,
            "  - {} {} rows={} sample={}",
            str_at(state, "surface"),
            str_at(state, "check_status"),
            int_at(state, "row_count"),
            sample
        );
    }
}

fn aggregate_problem_row(surface: &str, row: &Value) -> bool {
    match surface {
        "outbox" => {
            !str_at(row, "last_error").is_empty()
                || matches!(
                    str_at(row, "state"),
                    "pending" | "queued" | "sending" | "failed" | "error" | "rejected" | ""
                )
        }
        _ => false,
    }
}

fn render_outbox_problem(out: &mut String, row: &Value) {
    let resource = if str_at(row, "resource_key").is_empty() {
        format!("outbox/{}", int_at(row, "local_id"))
    } else {
        str_at(row, "resource_key").to_string()
    };
    let source = str_at(row, "source_ref");
    let source = if source.is_empty() { "-" } else { source };
    let _ = writeln!(
        out,
        "  - {} state={} retries={} event={} source={}",
        resource,
        str_at(row, "state"),
        int_at(row, "retries"),
        clipped(str_at(row, "event_id"), 12),
        source
    );
    if !str_at(row, "last_error").is_empty() {
        let _ = writeln!(out, "      error: {}", str_at(row, "last_error"));
    }
}

fn render_row(out: &mut String, surface: &str, row: &Value) {
    match surface {
        "subscriptions" => {
            let _ = writeln!(
                out,
                "  - refcount={} owners={}",
                int_at(row, "refcount"),
                array_len(row, "owners")
            );
        }
        "turn_lifecycle" => {
            let _ = writeln!(
                out,
                "  - working={} started={} transcript={}",
                bool_at(row, "working"),
                int_at(row, "turn_started_at"),
                str_at(row, "transcript_ref")
            );
        }
        "cursor" => {
            let _ = writeln!(
                out,
                "  - cursor={} last_frame={} delta_since={}",
                int_at(row, "cursor"),
                str_at(row, "last_frame"),
                int_at(row, "delta_since")
            );
        }
        "hook_context" => {
            let _ = writeln!(
                out,
                "  - revision={} nodes={} renders={} text_len={}",
                int_at(row, "revision"),
                int_at(row, "nodes"),
                int_at(row, "render_count"),
                str_at(row, "text").len()
            );
        }
        _ => {}
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

fn array_len(v: &Value, key: &str) -> usize {
    v.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

fn clipped(s: &str, max_chars: usize) -> String {
    let clipped = s.chars().take(max_chars).collect::<String>();
    if s.chars().count() > max_chars {
        format!("{clipped}...")
    } else {
        clipped
    }
}
