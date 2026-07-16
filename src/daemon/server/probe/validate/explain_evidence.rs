use super::report::{explain_found, explain_summary, push_check};
use super::target_checks::TargetChecks;
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn why(state: &Arc<DaemonState>, handle: Option<&str>) -> Option<Value> {
    match handle {
        Some(handle) => {
            match super::super::why::why_value(state, &json!({ "verb": "why", "handle": handle })) {
                Ok(v) => Some(v),
                Err(e) => Some(json!({
                    "verb": "why",
                    "handle": handle,
                    "found": false,
                    "error": e.to_string(),
                    "note": e.to_string(),
                })),
            }
        }
        None => None,
    }
}

pub(super) fn explanation(
    state: &Arc<DaemonState>,
    handle: &Option<crate::explain::Handle>,
) -> (Option<Value>, Option<String>) {
    match handle {
        Some(handle) => match state.with_store(|s| crate::explain::explain(s, handle)) {
            Ok(v) => (Some(v), None),
            Err(e) => (None, Some(e.to_string())),
        },
        None => (None, None),
    }
}

pub(super) fn push_checks(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    explanation: Option<&Value>,
    explain_error: Option<&str>,
    target_checks: &TargetChecks,
) {
    if let Some(v) = explanation {
        let explain_status = if explain_found(v) {
            "passed"
        } else if target_checks.event_checked() || target_checks.txn_checked() {
            "not_proven"
        } else {
            "failed"
        };
        push_check(checks, "explain", explain_status, explain_summary(v));
        if explain_status == "not_proven" {
            limitations.push("no Trellis receipt is recorded for this target".to_string());
        }
    }
    if let Some(error) = explain_error {
        push_check(checks, "explain", "failed", error.to_string());
    }
}
