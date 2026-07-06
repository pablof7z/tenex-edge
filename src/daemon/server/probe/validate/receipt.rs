//! Receipt target validation for `receipt:<id>` handles.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn receipt_target(target: &str) -> Option<i64> {
    target.strip_prefix("receipt:")?.parse().ok()
}

pub(super) fn receipt_evidence(state: &Arc<DaemonState>, target: &str, id: i64) -> Value {
    let result = state.with_store(|s| {
        let receipt = s.get_receipt(id)?;
        let commits = match &receipt {
            Some(row) => s.commits_for_surface_transaction(&row.surface, row.transaction_id)?,
            None => Vec::new(),
        };
        let artifact_receipts = match receipt.as_ref().and_then(|row| row.artifact_ref.as_deref()) {
            Some(artifact) => s.receipts_by_artifact_ref(artifact)?.len(),
            None => 0,
        };
        Ok::<_, anyhow::Error>((receipt, commits, artifact_receipts))
    });
    let (receipt, commits, artifact_receipt_count) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "receipt_id": id,
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": "receipt evidence could not read durable ledgers",
                "reason": e.to_string(),
            });
        }
    };
    let Some(receipt) = receipt else {
        return json!({
            "target": target,
            "receipt_id": id,
            "supported": true,
            "found": false,
            "commit_count": 0,
            "matching_commit_count": 0,
            "summary": format!("receipt `{id}` was not found"),
            "reason": "no receipts row exists for this id",
        });
    };

    let nearest_commit = nearest_commit(&commits, receipt.created_at);
    let matching_commit_count = commits
        .iter()
        .filter(|row| row.revision == receipt.revision)
        .count();
    let revision_matches_commit =
        nearest_commit.is_some_and(|row| row.revision == receipt.revision);
    let changed_summary_valid = serde_json::from_str::<Value>(&receipt.changed_summary).is_ok();
    let command_count = command_count(&receipt.commands);
    let commands_valid = command_count.is_some();
    let payload_valid = changed_summary_valid && commands_valid;
    let ok = revision_matches_commit && payload_valid;

    json!({
        "target": target,
        "receipt_id": id,
        "supported": true,
        "found": true,
        "surface": receipt.surface,
        "transaction_id": receipt.transaction_id,
        "revision": receipt.revision,
        "artifact_ref": receipt.artifact_ref,
        "created_at": receipt.created_at,
        "changed_summary_valid": changed_summary_valid,
        "commands_valid": commands_valid,
        "command_count": command_count.unwrap_or(0),
        "artifact_receipt_count": artifact_receipt_count,
        "commit_count": commits.len(),
        "matching_commit_count": matching_commit_count,
        "nearest_commit": nearest_commit.map(commit_json),
        "commit_delta_ms": nearest_commit.map(|row| (row.created_at - receipt.created_at).abs()),
        "revision_matches_commit": revision_matches_commit,
        "ok": ok,
        "summary": summary(id, ok, revision_matches_commit, payload_valid),
        "reason": reason(revision_matches_commit, payload_valid),
    })
}

pub(super) fn push_receipt_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty() {
        "failed"
    } else if bool_at(evidence, "ok") {
        "passed"
    } else if bool_at(evidence, "found") {
        "failed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "receipt_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
    if status == "passed" && str_at(evidence, "artifact_ref").is_empty() {
        limitations.push("receipt has no artifact_ref to follow into event validation".into());
    }
}

fn nearest_commit<'a>(
    commits: &'a [crate::state::trellis_commits::CommitRow],
    receipt_created_at: i64,
) -> Option<&'a crate::state::trellis_commits::CommitRow> {
    commits
        .iter()
        .min_by_key(|row| (row.created_at - receipt_created_at).abs())
}

fn command_count(commands: &str) -> Option<usize> {
    serde_json::from_str::<Value>(commands)
        .ok()
        .and_then(|v| v.as_array().map(Vec::len))
}

fn commit_json(row: &crate::state::trellis_commits::CommitRow) -> Value {
    json!({
        "id": row.id,
        "revision": row.revision,
        "mode": row.mode,
        "trigger_kind": row.trigger_kind,
        "trigger_ref": row.trigger_ref,
        "command_count": row.command_count,
        "output_count": row.output_count,
        "effect_count": row.effect_count,
        "suppressed_count": row.suppressed_count,
        "noop": row.noop != 0,
        "oracle_status": row.oracle_status,
        "oracle_error": row.oracle_error,
        "graph_nodes": row.graph_nodes,
        "graph_resources": row.graph_resources,
        "created_at": row.created_at,
    })
}

fn summary(id: i64, ok: bool, revision_matches_commit: bool, payload_valid: bool) -> String {
    if ok {
        format!("receipt `{id}` matches a durable commit and has valid payload JSON")
    } else if !revision_matches_commit {
        format!("receipt `{id}` has no matching commit revision")
    } else if !payload_valid {
        format!("receipt `{id}` has invalid payload JSON")
    } else {
        format!("receipt `{id}` could not be validated")
    }
}

fn reason(revision_matches_commit: bool, payload_valid: bool) -> &'static str {
    if !revision_matches_commit {
        "receipt exists, but the all-commit ledger has no nearest row at the same revision"
    } else if !payload_valid {
        "receipt changed_summary or commands payload is not valid JSON"
    } else {
        ""
    }
}
