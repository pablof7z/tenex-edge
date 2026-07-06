use super::report::{drift_surfaces, push_check};
use super::state_check::{all_surface_state_checks, target_state_evidence};
use super::target_checks::TargetChecks;
use super::{durable_outbox, session_consistency, DaemonState};
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) struct ResourceStateEvidence {
    pub(super) stats: Option<Value>,
    pub(super) stats_error: Option<String>,
    pub(super) surface_state: Option<Value>,
    pub(super) state_evidence: Option<Value>,
    pub(super) session_consistency: Option<Value>,
    pub(super) surface_states: Vec<Value>,
    pub(super) state_error: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect(
    state: &Arc<DaemonState>,
    params: &Value,
    surface: Option<&str>,
    global_seams_checked: bool,
    has_malformed_target: bool,
    target_checks: &TargetChecks,
    handle: Option<&str>,
    why: Option<&Value>,
    global_validation: bool,
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
) -> ResourceStateEvidence {
    let (stats, stats_error) = stats_evidence(state, params, surface, global_seams_checked, checks);
    let (surface_state, state_error) = surface_state_evidence(state, surface, has_malformed_target);
    if let Some(v) = &surface_state {
        durable_outbox::push_state_check(checks, v, handle, why, target_checks);
    }
    if let Some(error) = &state_error {
        push_check(checks, "state", "failed", error.clone());
    }
    let state_evidence = if target_checks.supported() {
        None
    } else {
        surface_state
            .as_ref()
            .and_then(|v| target_state_evidence(v, handle, why))
    };
    let (state_checks, surface_states) = if global_validation {
        all_surface_state_checks(state)
    } else {
        (Vec::new(), Vec::new())
    };
    checks.extend(state_checks);
    let session_consistency = if global_validation {
        let evidence = session_consistency::session_consistency_evidence(state);
        session_consistency::push_session_consistency_check(checks, limitations, &evidence);
        Some(evidence)
    } else {
        None
    };

    ResourceStateEvidence {
        stats,
        stats_error,
        surface_state,
        state_evidence,
        session_consistency,
        surface_states,
        state_error,
    }
}

fn stats_evidence(
    state: &Arc<DaemonState>,
    params: &Value,
    surface: Option<&str>,
    global_seams_checked: bool,
    checks: &mut Vec<Value>,
) -> (Option<Value>, Option<String>) {
    if surface.is_none() && !global_seams_checked {
        return (None, None);
    }
    let since = params.get("since").and_then(Value::as_i64).unwrap_or(0);
    match state.with_store(|s| super::super::stats::stats_value(s, surface, since)) {
        Ok(v) => {
            let drift = drift_surfaces(&v);
            push_check(
                checks,
                "resource_accounting",
                if drift.is_empty() { "passed" } else { "failed" },
                if drift.is_empty() {
                    "no resource drift in commit ledger".to_string()
                } else {
                    format!("resource drift: {}", drift.join(", "))
                },
            );
            (Some(v), None)
        }
        Err(e) => {
            let error = e.to_string();
            push_check(checks, "resource_accounting", "failed", error.clone());
            (None, Some(error))
        }
    }
}

fn surface_state_evidence(
    state: &Arc<DaemonState>,
    surface: Option<&str>,
    has_malformed_target: bool,
) -> (Option<Value>, Option<String>) {
    if has_malformed_target {
        return (None, None);
    }
    let Some(surface) = surface else {
        return (None, None);
    };
    match super::super::state::state_value(state, &json!({ "verb": "state", "surface": surface })) {
        Ok(v) => {
            let (status, summary) = super::state_check::state_check_summary(&v, None, None);
            (
                Some(super::state_check::annotated_surface_state(
                    v, status, &summary,
                )),
                None,
            )
        }
        Err(e) => (None, Some(e.to_string())),
    }
}
