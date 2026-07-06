use super::report::receipt_surface;
use super::target::{surface_from_explain_handle, surface_from_probe_handle, surface_target};
use super::target_checks::TargetChecks;
use crate::daemon::server::DaemonState;
use serde_json::Value;
use std::sync::Arc;

pub(super) fn selected_surface(
    target: Option<&str>,
    explain_handle: Option<&crate::explain::Handle>,
    target_checks: &TargetChecks,
    explanation: Option<&Value>,
    cause_label_evidence: Option<&Value>,
    fact_surface: Option<&str>,
    capsule_surface: Option<String>,
) -> Option<String> {
    target
        .and_then(surface_target)
        .map(str::to_string)
        .or_else(|| {
            target
                .and_then(surface_from_probe_handle)
                .map(str::to_string)
        })
        .or_else(|| {
            explain_handle
                .and_then(surface_from_explain_handle)
                .map(str::to_string)
        })
        .or_else(|| target_checks.surface_hint().map(str::to_string))
        .or_else(|| explanation.and_then(receipt_surface))
        .or_else(|| {
            cause_label_evidence
                .and_then(|v| v.get("surface").and_then(Value::as_str))
                .map(str::to_string)
        })
        .or_else(|| fact_surface.map(str::to_string))
        .or(capsule_surface)
}

pub(super) fn stored_capsule_surface(state: &Arc<DaemonState>, capsule: &str) -> Option<String> {
    let id = capsule.parse::<i64>().ok()?;
    state
        .with_store(|s| s.get_replay_capsule(id))
        .ok()
        .flatten()
        .map(|row| row.surface)
        .filter(|surface| !surface.trim().is_empty())
}
