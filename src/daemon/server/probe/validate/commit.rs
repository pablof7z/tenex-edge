//! Direct all-commit ledger row validation for `commit:<id>` targets.

use super::report::{bool_at, int_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn commit_target(target: &str) -> Option<i64> {
    target
        .strip_prefix("commit:")
        .or_else(|| target.strip_prefix("commit/"))
        .or_else(|| target.strip_prefix("trellis_commit:"))
        .or_else(|| target.strip_prefix("trellis_commit/"))
        .and_then(|rest| rest.split('/').next())
        .and_then(|id| id.parse().ok())
}

pub(super) fn commit_evidence(state: &Arc<DaemonState>, target: &str, id: i64) -> Value {
    let result = state.with_store(|s| {
        let commit = s.get_commit(id)?;
        let receipts = match &commit {
            Some(row) => s.receipts_for_surface_transaction(&row.surface, row.transaction_id)?,
            None => Vec::new(),
        };
        Ok::<_, anyhow::Error>((commit, receipts))
    });
    let (commit, receipts) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "commit_id": id,
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": "commit evidence could not read durable ledgers",
                "reason": e.to_string(),
            });
        }
    };
    let Some(commit) = commit else {
        return json!({
            "target": target,
            "commit_id": id,
            "supported": true,
            "found": false,
            "summary": format!("commit `{id}` was not found"),
            "reason": "no trellis_commits row exists for this id",
        });
    };

    let command_json_count = array_count(&commit.resource_commands_json);
    let output_json_count = array_count(&commit.output_frames_json);
    let payload_valid = array_count(&commit.changed_inputs_json).is_some()
        && array_count(&commit.changed_derived_json).is_some()
        && array_count(&commit.changed_collections_json).is_some()
        && command_json_count.is_some()
        && output_json_count.is_some();
    let command_count_matches = command_json_count == Some(commit.command_count);
    let output_count_matches = output_json_count == Some(commit.output_count);
    let candidate_receipts = receipts
        .iter()
        .filter(|row| row.revision == commit.revision)
        .collect::<Vec<_>>();
    let receipt_delta_ms = candidate_receipts
        .iter()
        .map(|row| (row.created_at - commit.created_at).abs())
        .min();
    let matching_receipts = candidate_receipts
        .iter()
        .filter(|row| {
            receipt_delta_ms
                .map(|delta| (row.created_at - commit.created_at).abs() == delta)
                .unwrap_or(false)
        })
        .take(5)
        .map(|row| receipt_json(row))
        .collect::<Vec<_>>();
    let ok = payload_valid && command_count_matches && output_count_matches;

    json!({
        "target": target,
        "commit_id": id,
        "supported": true,
        "found": true,
        "surface": commit.surface,
        "transaction_id": commit.transaction_id,
        "revision": commit.revision,
        "mode": commit.mode,
        "trigger_kind": commit.trigger_kind,
        "trigger_ref": commit.trigger_ref,
        "command_count": commit.command_count,
        "output_count": commit.output_count,
        "effect_count": commit.effect_count,
        "suppressed_count": commit.suppressed_count,
        "noop": commit.noop != 0,
        "oracle_status": commit.oracle_status,
        "oracle_error": commit.oracle_error,
        "duration_us": commit.duration_us,
        "graph_nodes": commit.graph_nodes,
        "graph_resources": commit.graph_resources,
        "created_at": commit.created_at,
        "payload_valid": payload_valid,
        "command_json_count": command_json_count.unwrap_or(-1),
        "output_json_count": output_json_count.unwrap_or(-1),
        "command_count_matches": command_count_matches,
        "output_count_matches": output_count_matches,
        "candidate_receipt_count": candidate_receipts.len(),
        "matching_receipt_count": matching_receipts.len(),
        "receipt_delta_ms": receipt_delta_ms,
        "receipt_match_scope": "nearest surface/transaction/revision",
        "receipts": matching_receipts,
        "ok": ok,
        "summary": summary(id, ok, &commit.surface, commit.transaction_id, commit.revision),
        "reason": reason(payload_valid, command_count_matches, output_count_matches),
    })
}

pub(super) fn push_commit_check(
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
        "name": "commit_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
    if status == "passed" && int_at(evidence, "matching_receipt_count") == 0 {
        limitations.push(
            "commit has no matching effect receipt; all-commit ledger is the explanation".into(),
        );
    }
}

fn array_count(raw: &str) -> Option<i64> {
    serde_json::from_str::<Value>(raw)
        .ok()
        .and_then(|v| v.as_array().map(|rows| rows.len() as i64))
}

fn receipt_json(row: &crate::state::receipts::ReceiptRow) -> Value {
    json!({
        "id": row.id,
        "revision": row.revision,
        "artifact_ref": row.artifact_ref,
        "created_at": row.created_at,
    })
}

fn summary(id: i64, ok: bool, surface: &str, transaction_id: i64, revision: i64) -> String {
    if ok {
        format!("commit `{id}` is a valid `{surface}` txn {transaction_id} rev {revision}")
    } else {
        format!("commit `{id}` has inconsistent ledger payload")
    }
}

fn reason(
    payload_valid: bool,
    command_count_matches: bool,
    output_count_matches: bool,
) -> &'static str {
    if !payload_valid {
        "commit ledger payload JSON is not valid array data"
    } else if !command_count_matches {
        "commit command_count does not match resource_commands_json"
    } else if !output_count_matches {
        "commit output_count does not match output_frames_json"
    } else {
        ""
    }
}
