//! Human render for `probe state`.

use serde_json::Value;
use std::fmt::Write as _;

fn str_at<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

fn i64_at(v: &Value, k: &str) -> i64 {
    v.get(k).and_then(Value::as_i64).unwrap_or(0)
}

fn strs(v: &Value, k: &str) -> Vec<String> {
    v.get(k)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// `probe state <surface>` — live per-surface values.
pub(super) fn render_state(v: &Value) -> String {
    let mut out = String::new();
    let surface = str_at(v, "surface");
    let _ = writeln!(out, "state {surface}  (live)");
    let empty = Vec::new();
    let rows = v.get("rows").and_then(Value::as_array).unwrap_or(&empty);
    if rows.is_empty() {
        let _ = writeln!(out, "  (none)");
        if let Some(note) = v.get("note").and_then(Value::as_str) {
            let _ = writeln!(out, "  {note}");
        }
    }
    for r in rows {
        match surface {
            "status" => render_status_row(&mut out, r),
            "hook_context" => render_hook_row(&mut out, r),
            _ => render_subscription_row(&mut out, r),
        }
    }
    out
}

fn render_status_row(out: &mut String, r: &Value) {
    let _ = writeln!(
        out,
        "  {:<10} {:<6} title={:?}  activity={:?}  channels={:?}",
        str_at(r, "session"),
        if r.get("busy").and_then(Value::as_bool) == Some(true) {
            "busy"
        } else {
            "idle"
        },
        str_at(r, "title"),
        str_at(r, "activity"),
        strs(r, "channels"),
    );
}

fn render_hook_row(out: &mut String, r: &Value) {
    let _ = writeln!(
        out,
        "  {:<18} rev {}  nodes {}  renders {}",
        str_at(r, "session"),
        i64_at(r, "revision"),
        i64_at(r, "nodes"),
        i64_at(r, "render_count"),
    );
    let causes = strs(r, "why_input_causes");
    if !causes.is_empty() {
        let _ = writeln!(out, "      caused by: {}", causes.join(", "));
    }
    let inputs = strs(r, "input_labels");
    if !inputs.is_empty() {
        let _ = writeln!(out, "      inputs:    {}", inputs.join(", "));
    }
    if let Some(text) = r.get("text").and_then(Value::as_str) {
        let first = text.lines().next().unwrap_or("");
        let _ = writeln!(out, "      text:      {first:?}");
    }
    if let Some(dump) = r.get("debug_dump").and_then(Value::as_str) {
        let _ = writeln!(out, "      dump:\n{dump}");
    }
}

fn render_subscription_row(out: &mut String, r: &Value) {
    let _ = writeln!(
        out,
        "  {:<18} refcount {}   owners: {}",
        str_at(r, "resource_key"),
        i64_at(r, "refcount"),
        strs(r, "owners").join(", "),
    );
}
