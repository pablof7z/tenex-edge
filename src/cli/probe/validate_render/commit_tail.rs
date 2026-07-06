//! Commit-ledger evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "commit evidence");
    if !bool_at(evidence, "found") {
        let _ = writeln!(
            out,
            "  - commit/{}: not found",
            int_at(evidence, "commit_id")
        );
        if !str_at(evidence, "reason").is_empty() {
            let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
        }
        return;
    }
    let _ = writeln!(
        out,
        "  - id={} {}:{} rev={} mode={} trigger={}:{}",
        int_at(evidence, "commit_id"),
        str_at(evidence, "surface"),
        int_at(evidence, "transaction_id"),
        int_at(evidence, "revision"),
        str_at(evidence, "mode"),
        str_at(evidence, "trigger_kind"),
        str_at(evidence, "trigger_ref")
    );
    let _ = writeln!(
        out,
        "  - commands={}/{} outputs={}/{} effects={} suppressed={} noop={}",
        int_at(evidence, "command_json_count"),
        int_at(evidence, "command_count"),
        int_at(evidence, "output_json_count"),
        int_at(evidence, "output_count"),
        int_at(evidence, "effect_count"),
        int_at(evidence, "suppressed_count"),
        bool_at(evidence, "noop")
    );
    let _ = writeln!(
        out,
        "  - payload_valid={} receipts={}/{} delta_ms={} oracle={} graph_nodes={} graph_resources={} at={}",
        bool_at(evidence, "payload_valid"),
        int_at(evidence, "matching_receipt_count"),
        int_at(evidence, "candidate_receipt_count"),
        int_at(evidence, "receipt_delta_ms"),
        str_at(evidence, "oracle_status"),
        int_at(evidence, "graph_nodes"),
        int_at(evidence, "graph_resources"),
        int_at(evidence, "created_at")
    );
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
