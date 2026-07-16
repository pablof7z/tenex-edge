//! `explain <handle>` query engine: point at an artifact, get what produced it.
//!
//! The store is opened only by the daemon, so the CLI reaches this through the
//! `explain` RPC; the engine itself is a pure `&Store` query so it is unit-testable
//! against a temp DB. It parses a `scheme:value` handle, then joins the
//! `receipts` ledger. For a published event it finds receipts by `artifact_ref`;
//! for a session it selects status receipts carrying that session's pubkey.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use crate::state::receipts::ReceiptRow;
use crate::state::Store;

/// How many rows a fallback scan (surface-scoped lookups) will examine.
const SCAN_LIMIT: u32 = 500;

/// A parsed `scheme:value` explain handle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Handle {
    /// A published nostr event id (e.g. a kind:30315).
    Event(String),
    /// A session, optionally at a timestamp (`session:<id>@<ts>`).
    Session { id: String, at: Option<i64> },
    /// A hook-context render for a session, optionally at a timestamp.
    Hook { id: String, at: Option<i64> },
}

/// Parse a `scheme:value` handle. Event ids and session ids never contain a
/// colon, so a single `split_once(':')` cleanly separates the scheme.
pub fn parse_handle(s: &str) -> Result<Handle> {
    let (scheme, value) = s
        .split_once(':')
        .with_context(|| format!("handle must be scheme:value, got `{s}`"))?;
    let value = value.trim();
    Ok(match scheme {
        "event" => Handle::Event(value.to_string()),
        "session" => {
            let (id, at) = split_at(value)?;
            Handle::Session { id, at }
        }
        "hook" => {
            let (id, at) = split_at(value)?;
            Handle::Hook { id, at }
        }
        other => bail!("unknown handle scheme `{other}` (event|session|hook)"),
    })
}

/// Split `id[@ts]` into its parts.
fn split_at(value: &str) -> Result<(String, Option<i64>)> {
    match value.split_once('@') {
        Some((id, ts)) => Ok((
            id.to_string(),
            Some(ts.parse().context("@<ts> must be unix millis")?),
        )),
        None => Ok((value.to_string(), None)),
    }
}

/// Resolve a handle against the ledgers into a joined record. This is the query
/// layer the RPC calls; the CLI renders the returned value (human or `--json`).
pub fn explain(store: &Store, handle: &Handle) -> Result<Value> {
    match handle {
        Handle::Event(id) => {
            let receipts = store.receipts_by_artifact_ref_prefix(id)?;
            Ok(record("event", receipts))
        }
        Handle::Session { id, at } => {
            let mut receipts = store
                .latest_receipts_for_surface("status", SCAN_LIMIT)?
                .into_iter()
                .filter(|receipt| receipt_pubkey(receipt).as_deref() == Some(id.as_str()))
                .collect::<Vec<_>>();
            select_near(&mut receipts, *at);
            receipts.truncate(1);
            Ok(record("session", receipts))
        }
        Handle::Hook { id, at } => {
            let rows = match at {
                Some(ts) => store
                    .find_hook_receipt_for_pubkey_near(id, *ts)?
                    .into_iter()
                    .collect(),
                None => store.latest_hook_receipts_for_pubkey(id, 1)?,
            };
            Ok(record("hook", rows))
        }
    }
}

fn receipt_pubkey(r: &ReceiptRow) -> Option<String> {
    serde_json::from_str::<Value>(&r.changed_summary)
        .ok()?
        .get("pubkey")?
        .as_str()
        .map(str::to_string)
}

/// Order receipts by proximity to `at`; use newest-first when no timestamp is
/// supplied.
fn select_near(rows: &mut [ReceiptRow], at: Option<i64>) {
    match at {
        Some(ts) => rows.sort_by_key(|row| (row.created_at - ts).abs()),
        None => rows.sort_by_key(|row| std::cmp::Reverse(row.created_at)),
    }
}

/// Assemble the joined record the RPC returns and the CLI renders.
fn record(kind: &str, receipts: Vec<ReceiptRow>) -> Value {
    json!({
        "kind": kind,
        "receipts": receipts.iter().map(receipt_json).collect::<Vec<_>>(),
    })
}

/// A receipt row as plain JSON for the CLI renderer.
pub fn receipt_json(r: &ReceiptRow) -> Value {
    json!({
        "id": r.id,
        "surface": r.surface,
        "transaction_id": r.transaction_id,
        "revision": r.revision,
        "changed_summary": r.changed_summary,
        "commands": r.commands,
        "artifact_ref": r.artifact_ref,
        "created_at": r.created_at,
    })
}

#[cfg(test)]
mod tests;
