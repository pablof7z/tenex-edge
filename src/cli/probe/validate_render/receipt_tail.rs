//! Receipt evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "receipt evidence");
    let artifact = evidence
        .get("artifact_ref")
        .and_then(Value::as_str)
        .unwrap_or("(none)");
    let _ = writeln!(
        out,
        "  - id={} surface={} txn={} rev={} at={} artifact={}",
        int_at(evidence, "receipt_id"),
        str_at(evidence, "surface"),
        int_at(evidence, "transaction_id"),
        int_at(evidence, "revision"),
        int_at(evidence, "created_at"),
        artifact
    );
    let _ = writeln!(
        out,
        "  - commit_match={} matches={} total_commits={} delta_ms={} payload_valid={} commands={} artifact_receipts={}",
        bool_at(evidence, "revision_matches_commit"),
        int_at(evidence, "matching_commit_count"),
        int_at(evidence, "commit_count"),
        int_at(evidence, "commit_delta_ms"),
        bool_at(evidence, "changed_summary_valid") && bool_at(evidence, "commands_valid"),
        int_at(evidence, "command_count"),
        int_at(evidence, "artifact_receipt_count")
    );
    if let Some(commit) = evidence.get("nearest_commit").filter(|v| !v.is_null()) {
        let _ = writeln!(
            out,
            "  - nearest commit id={} rev={} at={} mode={} trigger={}:{} effects={} commands={} noop={}",
            int_at(commit, "id"),
            int_at(commit, "revision"),
            int_at(commit, "created_at"),
            str_at(commit, "mode"),
            str_at(commit, "trigger_kind"),
            str_at(commit, "trigger_ref"),
            int_at(commit, "effect_count"),
            int_at(commit, "command_count"),
            bool_at(commit, "noop")
        );
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
