//! Error evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, v: &Value) {
    const ERRORS: [(&str, &str); 6] = [
        ("stats", "stats_error"),
        ("state", "state_error"),
        ("explain", "explain_error"),
        ("simulate", "simulate_error"),
        ("acid", "acid_error"),
        ("replay", "replay_error"),
    ];
    let rows = ERRORS
        .iter()
        .filter_map(|(label, key)| {
            v.get(*key)
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(|error| (*label, error))
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "error evidence");
    for (label, error) in rows {
        let _ = writeln!(out, "  - {label}: {error}");
    }
}
