//! Subscription evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "subscription evidence");
    let _ = writeln!(
        out,
        "  - {} ({}) resources={}/{} receipts={} revision={}",
        str_at(evidence, "entity"),
        str_at(evidence, "kind"),
        int_at(evidence, "found_resource_count"),
        int_at(evidence, "expected_resource_count"),
        int_at(evidence, "receipt_count"),
        int_at(evidence, "revision")
    );
    let empty = Vec::new();
    for resource in evidence
        .get("resources")
        .and_then(Value::as_array)
        .unwrap_or(&empty)
    {
        let status = if bool_at(resource, "found") {
            "live"
        } else {
            "missing"
        };
        let _ = writeln!(
            out,
            "  - {}: {} refcount={} owners={}",
            str_at(resource, "resource_key"),
            status,
            int_at(resource, "refcount"),
            strings(resource.get("owners")).join(",")
        );
        let causes = strings(resource.get("input_causes"));
        if !causes.is_empty() {
            let _ = writeln!(out, "    causes={}", causes.join(","));
        }
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn strings(value: Option<&Value>) -> Vec<&str> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
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
