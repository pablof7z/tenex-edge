//! Transaction evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "transaction evidence");
    let _ = writeln!(
        out,
        "  - {}:{} commits={} receipts={} total_commits={} total_receipts={} revisions_match={} ambiguous={}",
        str_at(evidence, "surface"),
        int_at(evidence, "transaction_id"),
        int_at(evidence, "commit_count"),
        int_at(evidence, "receipt_count"),
        int_at(evidence, "total_commit_count"),
        int_at(evidence, "total_receipt_count"),
        bool_at(evidence, "receipt_revisions_match_commits"),
        bool_at(evidence, "ambiguous")
    );
    if let Some(at) = evidence.get("at").and_then(Value::as_i64) {
        let _ = writeln!(
            out,
            "  - selected_at={at} delta_ms={}",
            int_at(evidence, "at_delta_ms")
        );
    }
    if let Some(commit) = evidence.get("latest_commit").filter(|v| !v.is_null()) {
        let _ = writeln!(
            out,
            "  - commit id={} rev={} at={} mode={} trigger={}:{} effects={} commands={} outputs={} noop={}",
            int_at(commit, "id"),
            int_at(commit, "revision"),
            int_at(commit, "created_at"),
            str_at(commit, "mode"),
            str_at(commit, "trigger_kind"),
            str_at(commit, "trigger_ref"),
            int_at(commit, "effect_count"),
            int_at(commit, "command_count"),
            int_at(commit, "output_count"),
            bool_at(commit, "noop")
        );
    }
    render_receipts(out, evidence.get("receipts"));
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn render_receipts(out: &mut String, value: Option<&Value>) {
    let receipts = value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    for receipt in receipts.iter().take(3) {
        let artifact = receipt
            .get("artifact_ref")
            .and_then(Value::as_str)
            .unwrap_or("(none)");
        let _ = writeln!(
            out,
            "  - receipt id={} rev={} artifact={}",
            int_at(receipt, "id"),
            int_at(receipt, "revision"),
            artifact
        );
    }
    if receipts.len() > 3 {
        let _ = writeln!(out, "  - ... {} more receipt(s)", receipts.len() - 3);
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
