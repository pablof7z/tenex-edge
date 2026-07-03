//! Retrospective instrumentation host-boundary helpers (Slice 8).
//!
//! The storage ledgers (`state::llm_calls`, `state::receipts`) are pure and never
//! read the clock. This module is the host-side glue that captures a distill
//! round-trip and each reconciler drive-seam receipt into those ledgers: it reads
//! the wall clock HERE, hashes the transcript window, flattens Trellis-vocabulary
//! `TransactionResult`s into plain JSON, and records — logging and continuing on a
//! failed insert so instrumentation never blocks the hot path.
//!
//! The `window_hash` is the join key: the SAME sha256 of the exact transcript
//! slice fed to the LLM is recorded on the `llm_calls` row AND carried in the
//! status receipt's `changed_summary`, so a published kind:30315 (looked up by
//! its event id → receipt) rejoins the exact LLM inputs that produced it.

use serde::Serialize;
use sha2::{Digest, Sha256};
use trellis_core::{NodeId, ResourceCommand};

use crate::state::llm_calls::NewLlmCall;
use crate::state::receipts::NewReceipt;
use crate::state::Store;

/// Host wall clock in unix milliseconds, read at the boundary (ledgers never do).
pub fn now_millis() -> i64 {
    crate::util::now_millis() as i64
}

/// Stable content pointer for a transcript slice: `sha256:<hex>`. This is the
/// join key between an `llm_calls` row and the status receipt of the 30315 it fed.
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

/// The verbatim distill LLM round-trip, captured at the model seam in `distill`
/// and completed host-side (session id, window hash, clock) before recording.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistillCapture {
    /// Distiller provider (`claude-cli` / `openrouter` / `ollama` / `command`).
    pub provider: String,
    /// Model name (or the external command string for the override seam).
    pub model: String,
    /// The system prompt actually sent to the model.
    pub system_prompt: String,
    /// The exact bytes fed as user input (incl. any `CURRENT TITLE:` prefix).
    pub transcript_slice: String,
    /// The raw response text before `parse_labels`.
    pub raw_response: String,
}

/// Record one distill round-trip. Best-effort: a failed insert is logged, not
/// propagated, so distillation is never blocked by instrumentation.
pub fn record_llm_call(
    store: &Store,
    session_id: &str,
    window_hash: &str,
    cap: &DistillCapture,
    parsed_title: Option<&str>,
    parsed_activity: Option<&str>,
    created_at: i64,
) {
    let row = NewLlmCall {
        session_id: session_id.to_string(),
        window_hash: window_hash.to_string(),
        provider: cap.provider.clone(),
        model: cap.model.clone(),
        system_prompt: cap.system_prompt.clone(),
        transcript_slice: cap.transcript_slice.clone(),
        raw_response: cap.raw_response.clone(),
        parsed_title: parsed_title.map(str::to_string),
        parsed_activity: parsed_activity.map(str::to_string),
        created_at,
    };
    if let Err(e) = store.record_llm_call(&row) {
        tracing::warn!(session = %session_id, error = %e, "record_llm_call failed — distill round-trip not instrumented");
    }
}

/// Record one flattened reconciler receipt. Best-effort like [`record_llm_call`].
pub fn record_receipt(store: &Store, row: NewReceipt) {
    if let Err(e) = store.record_receipt(&row) {
        tracing::warn!(surface = %row.surface, error = %e, "record_receipt failed — drive seam not instrumented");
    }
}

/// A changed node summary plus optional join context, as a compact JSON string.
/// Node identities are the graph-local numeric ids (Trellis-free strings); the
/// optional `session_id`/`window_hash` are the status-surface join keys.
pub fn changed_summary_json(
    inputs: &[NodeId],
    derived: &[NodeId],
    collections: &[NodeId],
    session_id: Option<&str>,
    window_hash: Option<&str>,
) -> String {
    #[derive(Serialize)]
    struct Summary<'a> {
        inputs: Vec<u64>,
        derived: Vec<u64>,
        collections: Vec<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        window_hash: Option<&'a str>,
    }
    let ids = |ns: &[NodeId]| ns.iter().map(|n| n.get()).collect::<Vec<_>>();
    serde_json::to_string(&Summary {
        inputs: ids(inputs),
        derived: ids(derived),
        collections: ids(collections),
        session_id,
        window_hash,
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
        let s = changed_summary_json(&[], &[], &[], Some("sid-1"), Some("sha256:ab"));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["session_id"], "sid-1");
        assert_eq!(v["window_hash"], "sha256:ab");
    }

    #[test]
    fn changed_summary_omits_absent_join_keys() {
        let s = changed_summary_json(&[], &[], &[], None, None);
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert!(v.get("session_id").is_none());
        assert!(v.get("window_hash").is_none());
    }
}
