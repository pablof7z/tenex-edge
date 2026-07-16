//! Retrospective instrumentation host-boundary helpers.
//!
//! Storage ledgers are pure and never read the clock. This module is the host-side
//! glue that hashes artifact bytes, flattens Trellis-vocabulary transaction
//! results into plain JSON, and records them without blocking the hot path.

use serde::Serialize;
use sha2::{Digest, Sha256};
use trellis_core::{NodeId, ResourceCommand};

use crate::reconcile::labels::CommitFacts;
use crate::state::receipts::NewReceipt;
use crate::state::trellis_commits::NewCommit;
use crate::state::Store;

/// Host wall clock in unix milliseconds, read at the boundary (ledgers never do).
pub fn now_millis() -> i64 {
    crate::util::now_millis() as i64
}

/// Stable content pointer for arbitrary bytes encoded as text: `sha256:<hex>`.
pub fn window_hash(slice: &str) -> String {
    let digest = Sha256::digest(slice.as_bytes());
    let mut hex = String::with_capacity(7 + digest.len() * 2);
    hex.push_str("sha256:");
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// Record one flattened reconciler receipt without blocking the reconciler.
pub fn record_receipt(store: &Store, row: NewReceipt) {
    if let Err(e) = store.record_receipt(&row) {
        tracing::warn!(surface = %row.surface, error = %e, "record_receipt failed — drive seam not instrumented");
    }
}

/// Record one commit into the all-commit ledger (§4.1) — the sibling of
/// [`record_receipt`], but for EVERY transaction (effectful or no-op). The caller
/// supplies the [`CommitFacts`] (label arrays + counts, already Trellis-free), the
/// drive `trigger_kind`, the host-measured `duration_us`, and `created_at`. Same
/// best-effort contract: a ledger insert failure is logged, never propagated, so
/// instrumentation can never block or fail the reconciler's effect.
pub fn record_commit(
    store: &Store,
    surface: &str,
    trigger_kind: &str,
    trigger_ref: Option<&str>,
    facts: &CommitFacts,
    duration_us: i64,
    created_at: i64,
) {
    let row = NewCommit {
        surface: surface.to_string(),
        transaction_id: facts.transaction_id,
        revision: facts.revision,
        mode: surface_mode(surface).to_string(),
        trigger_kind: trigger_kind.to_string(),
        trigger_ref: trigger_ref.unwrap_or_default().to_string(),
        changed_inputs_json: json_labels(&facts.changed_inputs),
        changed_derived_json: json_labels(&facts.changed_derived),
        changed_collections_json: json_labels(&facts.changed_collections),
        resource_commands_json: facts.resource_commands_json.clone(),
        output_frames_json: facts.output_frames_json.clone(),
        command_count: facts.command_count,
        output_count: facts.output_count,
        effect_count: facts.command_count + facts.output_count,
        suppressed_count: facts.noop as i64,
        noop: facts.noop as i64,
        oracle_status: None,
        oracle_error: None,
        duration_us,
        graph_nodes: facts.graph_nodes,
        graph_resources: facts.graph_resources,
        created_at,
    };
    if let Err(e) = store.record_commit(&row) {
        tracing::warn!(surface, error = %e, "record_commit failed — commit not ledgered");
    }
}

fn surface_mode(surface: &str) -> &'static str {
    match surface {
        "status" | "subscriptions" | "hook_context" | "delivery" => "authoritative",
        _ => "imperative",
    }
}

/// Serialize a label array to a compact JSON string, never failing the caller.
fn json_labels(labels: &[String]) -> String {
    serde_json::to_string(labels).unwrap_or_else(|_| "[]".into())
}

/// A changed node summary plus optional join context, as a compact JSON string.
/// Node identities are the graph-local numeric ids (Trellis-free strings); the
/// optional `pubkey` identifies the session for a status-surface receipt.
pub fn changed_summary_json(
    inputs: &[NodeId],
    derived: &[NodeId],
    collections: &[NodeId],
    pubkey: Option<&str>,
) -> String {
    #[derive(Serialize)]
    struct Summary<'a> {
        inputs: Vec<u64>,
        derived: Vec<u64>,
        collections: Vec<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pubkey: Option<&'a str>,
    }
    let ids = |ns: &[NodeId]| ns.iter().map(|n| n.get()).collect::<Vec<_>>();
    serde_json::to_string(&Summary {
        inputs: ids(inputs),
        derived: ids(derived),
        collections: ids(collections),
        pubkey,
    })
    .unwrap_or_else(|_| "{}".into())
}

/// The resource plan flattened to `[{kind,key,reason}]`. `reason` is the command
/// operation (open/close/replace/refresh) — a faithful, payload-free "why".
pub fn commands_json<C>(commands: &[ResourceCommand<C>]) -> String {
    #[derive(Serialize)]
    struct Cmd<'a> {
        kind: &'a str,
        key: &'a str,
        reason: &'a str,
    }
    let out: Vec<Cmd> = commands
        .iter()
        .map(|c| {
            let kind = match c {
                ResourceCommand::Open { .. } => "open",
                ResourceCommand::Close { .. } => "close",
                ResourceCommand::Replace { .. } => "replace",
                ResourceCommand::Refresh { .. } => "refresh",
            };
            Cmd {
                kind,
                key: c.key().as_str(),
                reason: kind,
            }
        })
        .collect();
    serde_json::to_string(&out).unwrap_or_else(|_| "[]".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_hash_is_stable_and_prefixed() {
        let a = window_hash("CURRENT TITLE: x\n\nTRANSCRIPT:\nuser: hi");
        let b = window_hash("CURRENT TITLE: x\n\nTRANSCRIPT:\nuser: hi");
        assert_eq!(a, b);
        assert!(a.starts_with("sha256:"));
        assert_eq!(a.len(), "sha256:".len() + 64);
        assert_ne!(a, window_hash("different"));
    }

    #[test]
    fn changed_summary_carries_join_keys() {
        let s = changed_summary_json(&[], &[], &[], Some("pubkey-1"));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["pubkey"], "pubkey-1");
    }

    #[test]
    fn changed_summary_omits_absent_join_keys() {
        let s = changed_summary_json(&[], &[], &[], None);
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert!(v.get("pubkey").is_none());
    }
}
