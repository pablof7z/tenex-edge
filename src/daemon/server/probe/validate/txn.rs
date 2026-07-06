//! Transaction target validation for `txn:<surface>:<id>[@<ts>]` handles.

use super::report::{bool_at, int_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) struct TxnTarget {
    pub(super) surface: String,
    pub(super) transaction_id: i64,
    pub(super) at: Option<i64>,
}

pub(super) fn txn_target(target: &str) -> Option<TxnTarget> {
    let rest = target.strip_prefix("txn:")?;
    let (surface, raw_id) = rest.split_once(':')?;
    let (id, at) = split_at(raw_id)?;
    Some(TxnTarget {
        surface: surface.to_string(),
        transaction_id: id.parse().ok()?,
        at,
    })
}

pub(super) fn txn_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    surface: &str,
    transaction_id: i64,
    at: Option<i64>,
) -> Value {
    let result = state.with_store(|s| {
        let commits = s.commits_for_surface_transaction(surface, transaction_id)?;
        let receipts = s.receipts_for_surface_transaction(surface, transaction_id)?;
        Ok::<_, anyhow::Error>((commits, receipts))
    });
    let (commits, receipts) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "surface": surface,
                "transaction_id": transaction_id,
                "at": at,
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": "transaction evidence could not read durable ledgers",
                "reason": e.to_string(),
            });
        }
    };
    let total_commit_count = commits.len();
    let total_receipt_count = receipts.len();
    let selected_commits = select_commits(&commits, at);
    let selected_receipts = select_receipts(&receipts, selected_commits.as_slice(), at);
    let commit_count = selected_commits.len();
    let receipt_count = selected_receipts.len();
    let latest_commit = selected_commits.first().map(|row| commit_json(row));
    let receipt_rows = receipts
        .iter()
        .filter(|row| {
            selected_receipts
                .iter()
                .any(|selected| selected.id == row.id)
        })
        .take(5)
        .map(receipt_json)
        .collect::<Vec<_>>();
    let revisions = selected_commits
        .iter()
        .map(|row| row.revision)
        .collect::<Vec<_>>();
    let receipts_match = selected_receipts.iter().all(|receipt| {
        revisions
            .iter()
            .any(|revision| *revision == receipt.revision)
    });
    let found = commit_count > 0 || receipt_count > 0;
    let ok = commit_count == 1 && receipts_match;
    let at_delta_ms = selected_commits
        .first()
        .and_then(|row| at.map(|at| (row.created_at - at).abs()));

    json!({
        "target": target,
        "surface": surface,
        "transaction_id": transaction_id,
        "at": at,
        "at_delta_ms": at_delta_ms,
        "supported": true,
        "found": found,
        "total_commit_count": total_commit_count,
        "total_receipt_count": total_receipt_count,
        "commit_count": commit_count,
        "receipt_count": receipt_count,
        "latest_commit": latest_commit,
        "receipts": receipt_rows,
        "receipt_revisions_match_commits": receipts_match,
        "ambiguous": commit_count > 1,
        "ok": ok,
        "summary": summary(surface, transaction_id, at, commit_count, receipt_count, receipts_match),
        "reason": reason(at, commit_count, receipt_count, receipts_match),
    })
}

pub(super) fn push_txn_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty() {
        "failed"
    } else if bool_at(evidence, "ok") {
        "passed"
    } else if int_at(evidence, "commit_count") == 0 && int_at(evidence, "receipt_count") > 0 {
        "failed"
    } else if !bool_at(evidence, "found") {
        "not_proven"
    } else if bool_at(evidence, "receipt_revisions_match_commits") {
        "not_proven"
    } else {
        "failed"
    };
    checks.push(json!({
        "name": "txn_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
    if status == "passed" && int_at(evidence, "receipt_count") == 0 {
        limitations
            .push("transaction has no effect receipt; commit ledger is the explanation".into());
    }
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

fn receipt_json(row: &crate::state::receipts::ReceiptRow) -> Value {
    json!({
        "id": row.id,
        "revision": row.revision,
        "artifact_ref": row.artifact_ref,
        "created_at": row.created_at,
    })
}

fn select_commits<'a>(
    rows: &'a [crate::state::trellis_commits::CommitRow],
    at: Option<i64>,
) -> Vec<&'a crate::state::trellis_commits::CommitRow> {
    let Some(at) = at else {
        return rows.iter().collect();
    };
    let Some(min_delta) = rows.iter().map(|row| (row.created_at - at).abs()).min() else {
        return Vec::new();
    };
    rows.iter()
        .filter(|row| (row.created_at - at).abs() == min_delta)
        .collect()
}

fn select_receipts<'a>(
    rows: &'a [crate::state::receipts::ReceiptRow],
    commits: &[&crate::state::trellis_commits::CommitRow],
    at: Option<i64>,
) -> Vec<&'a crate::state::receipts::ReceiptRow> {
    let mut candidates = rows.iter().collect::<Vec<_>>();
    let Some(at) = at else {
        return candidates;
    };
    if candidates.is_empty() {
        return candidates;
    }
    let anchor = commits.first().map(|row| row.created_at).unwrap_or(at);
    let min_delta = candidates
        .iter()
        .map(|row| (row.created_at - anchor).abs())
        .min()
        .unwrap_or(0);
    candidates.retain(|row| (row.created_at - anchor).abs() == min_delta);
    candidates
}

fn split_at(raw_id: &str) -> Option<(&str, Option<i64>)> {
    match raw_id.split_once('@') {
        Some((id, at)) => Some((id, Some(at.parse().ok()?))),
        None => Some((raw_id, None)),
    }
}

fn summary(
    surface: &str,
    transaction_id: i64,
    at: Option<i64>,
    commit_count: usize,
    receipt_count: usize,
    receipts_match: bool,
) -> String {
    if commit_count == 1 && receipts_match {
        match at {
            Some(at) => format!(
                "txn `{surface}:{transaction_id}@{at}` resolves to one durable commit and {receipt_count} receipt(s)"
            ),
            None => format!(
                "txn `{surface}:{transaction_id}` has durable commit evidence and {receipt_count} receipt(s)"
            ),
        }
    } else if commit_count > 1 {
        format!("txn `{surface}:{transaction_id}` appears in {commit_count} commit-ledger epoch(s)")
    } else if commit_count == 0 && receipt_count > 0 {
        format!("txn `{surface}:{transaction_id}` has receipt(s) but no all-commit ledger row")
    } else if commit_count == 1 {
        format!("txn `{surface}:{transaction_id}` receipt revisions do not match commit revision")
    } else {
        format!("txn `{surface}:{transaction_id}` was not found in durable ledgers")
    }
}

fn reason(
    at: Option<i64>,
    commit_count: usize,
    receipt_count: usize,
    receipts_match: bool,
) -> &'static str {
    if commit_count > 1 {
        if at.is_some() {
            "timestamp-qualified transaction handle still matches multiple commit rows"
        } else {
            "transaction id is graph-local and appears in multiple daemon epochs; add epoch/time context to disambiguate"
        }
    } else if commit_count == 0 && receipt_count > 0 {
        "receipt exists, but the all-commit ledger has no matching transaction row"
    } else if commit_count == 1 && !receipts_match {
        "one or more receipts have revisions that do not match the matching commit row"
    } else if commit_count == 0 {
        "no commit or receipt row was found for this transaction handle"
    } else {
        ""
    }
}
