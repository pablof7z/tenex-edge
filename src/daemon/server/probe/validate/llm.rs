//! LLM-call target validation.

use super::report::{bool_at, int_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn llm_target(target: &str) -> Option<i64> {
    target.strip_prefix("llm:")?.parse().ok()
}

pub(super) fn llm_evidence(state: &Arc<DaemonState>, target: &str, id: i64) -> Value {
    let result = state.with_store(|s| {
        let call = s.get_llm_call(id)?;
        let explanation = crate::explain::explain(s, &crate::explain::Handle::Llm(id))?;
        let session = match &call {
            Some(call) => s.get_session(&call.pubkey)?,
            None => None,
        };
        Ok::<_, anyhow::Error>((call, explanation, session))
    });
    let (call, explanation, session) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "llm_id": id,
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": "LLM evidence could not read local ledgers",
                "reason": e.to_string(),
            });
        }
    };
    let Some(call) = call else {
        return json!({
            "target": target,
            "llm_id": id,
            "supported": true,
            "found": false,
            "call_found": false,
            "receipt_count": 0,
            "summary": format!("llm call `{id}` was not found"),
            "reason": "no llm_calls row exists for this id",
        });
    };
    let receipts = explanation
        .get("receipts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let receipt_count = receipts.len();
    let session_row_found = session.is_some();
    let receipt_refs = receipts
        .iter()
        .filter_map(|r| r.get("artifact_ref").and_then(Value::as_str))
        .take(5)
        .map(str::to_string)
        .collect::<Vec<_>>();

    json!({
        "target": target,
        "llm_id": id,
        "supported": true,
        "found": true,
        "call_found": true,
        "pubkey": call.pubkey,
        "session_row_found": session_row_found,
        "session_alive": session.as_ref().is_some_and(|s| s.alive),
        "provider": call.provider,
        "model": call.model,
        "window_hash": call.window_hash,
        "created_at": call.created_at,
        "parsed_title": call.parsed_title,
        "parsed_activity": call.parsed_activity,
        "system_prompt_bytes": call.system_prompt.len(),
        "transcript_slice_bytes": call.transcript_slice.len(),
        "raw_response_bytes": call.raw_response.len(),
        "receipt_count": receipt_count,
        "receipt_artifacts": receipt_refs,
        "ok": true,
        "summary": summary(id, receipt_count, session_row_found),
        "reason": reason(receipt_count),
    })
}

pub(super) fn push_llm_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if !str_at(evidence, "error").is_empty() {
        "failed"
    } else if bool_at(evidence, "call_found") {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "llm_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if bool_at(evidence, "call_found") && int_at(evidence, "receipt_count") == 0 {
        limitations.push("LLM call has no status receipt joined by window_hash".to_string());
    }
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn summary(id: i64, receipt_count: usize, session_row_found: bool) -> String {
    if receipt_count > 0 && session_row_found {
        format!("llm call `{id}` exists and joins to {receipt_count} status receipt(s)")
    } else if receipt_count > 0 {
        format!("llm call `{id}` exists with receipt(s), but no local session row")
    } else {
        format!("llm call `{id}` exists without a joined status receipt")
    }
}

fn reason(receipt_count: usize) -> &'static str {
    if receipt_count == 0 {
        "no status receipt currently joins to this LLM call's window_hash"
    } else {
        ""
    }
}
