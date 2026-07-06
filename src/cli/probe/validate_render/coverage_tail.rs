//! Validation coverage evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    if str_at(evidence, "kind") == "validation_table" {
        render_table(out, evidence);
        return;
    }
    if str_at(evidence, "kind") == "validation_lookup" {
        render_lookup(out, evidence);
        return;
    }

    let _ = writeln!(out);
    let _ = writeln!(out, "validation coverage evidence");
    let _ = writeln!(
        out,
        "  - tables={}/{} direct={} aggregate={} uncovered={}",
        int_at(evidence, "covered_table_count"),
        int_at(evidence, "table_count"),
        int_at(evidence, "direct_table_count"),
        int_at(evidence, "aggregate_table_count"),
        array_len(evidence, "uncovered_tables")
    );
    if let Some(rows) = evidence.get("durable_tables").and_then(Value::as_array) {
        for row in rows.iter().take(8) {
            let _ = writeln!(
                out,
                "  - {} [{}] -> {}",
                str_at(row, "table"),
                str_at(row, "mode"),
                str_at(row, "targets")
            );
        }
        if rows.len() > 8 {
            let _ = writeln!(out, "  - ... {} more durable table(s)", rows.len() - 8);
        }
    }
    if let Some(rows) = evidence.get("surfaces").and_then(Value::as_array) {
        let modes = rows
            .iter()
            .map(|row| format!("{}={}", str_at(row, "surface"), str_at(row, "mode")))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(out, "  - surfaces: {modes}");
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn render_table(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "table coverage evidence");
    let _ = writeln!(
        out,
        "  - table={} present={} covered={} rows={} columns={}",
        str_at(evidence, "table"),
        bool_at(evidence, "present"),
        bool_at(evidence, "covered"),
        int_at(evidence, "row_count"),
        int_at(evidence, "column_count")
    );
    if !str_at(evidence, "targets").is_empty() {
        let _ = writeln!(
            out,
            "  - mode={} targets={} proves={}",
            str_at(evidence, "mode"),
            str_at(evidence, "targets"),
            str_at(evidence, "proves")
        );
    }
    if let Some(columns) = evidence.get("columns").and_then(Value::as_array) {
        let names = columns
            .iter()
            .filter_map(Value::as_str)
            .take(12)
            .collect::<Vec<_>>()
            .join(", ");
        if !names.is_empty() {
            let _ = writeln!(out, "  - columns: {names}");
        }
        if columns.len() > 12 {
            let _ = writeln!(out, "  - ... {} more column(s)", columns.len() - 12);
        }
    }
    if let Some(samples) = evidence.get("sample_targets").and_then(Value::as_array) {
        for sample in samples.iter().take(5) {
            let also = str_at(sample, "also");
            if also.is_empty() {
                let _ = writeln!(out, "  - sample: {}", str_at(sample, "target"));
            } else {
                let _ = writeln!(
                    out,
                    "  - sample: {} (also {})",
                    str_at(sample, "target"),
                    also
                );
            }
        }
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn render_lookup(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "lookup evidence");
    let _ = writeln!(
        out,
        "  - needle={} matches={}",
        str_at(evidence, "needle"),
        int_at(evidence, "match_count")
    );
    if let Some(matches) = evidence.get("matches").and_then(Value::as_array) {
        for row in matches.iter().take(8) {
            let also = str_at(row, "also");
            if also.is_empty() {
                let _ = writeln!(
                    out,
                    "  - {} -> {}",
                    str_at(row, "table"),
                    str_at(row, "target")
                );
            } else {
                let _ = writeln!(
                    out,
                    "  - {} -> {} (also {})",
                    str_at(row, "table"),
                    str_at(row, "target"),
                    also
                );
            }
        }
        if matches.len() > 8 {
            let _ = writeln!(out, "  - ... {} more match(es)", matches.len() - 8);
        }
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn str_at<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn int_at(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}

fn bool_at(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn array_len(v: &Value, key: &str) -> usize {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(0, |rows| rows.len())
}
