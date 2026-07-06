//! Generic validation checks for historical durable outbox rows.

use super::report::{bool_at, push_check, why_summary};
use super::state_check::state_check_summary;
use super::target_checks::TargetChecks;
use serde_json::Value;

pub(super) fn push_why_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    why: &Value,
    target_checks: &TargetChecks,
) {
    let status = if bool_at(why, "found") {
        "passed"
    } else if target_checks.historical_outbox_store_only()
        || target_checks.relay_status_without_graph()
    {
        "not_proven"
    } else {
        "failed"
    };
    push_check(checks, "why", status, why_summary(why));
    if status == "not_proven" && target_checks.historical_outbox_store_only() {
        limitations.push("durable outbox row has no live graph cause audit".to_string());
    } else if status == "not_proven" && target_checks.relay_status_without_graph() {
        limitations.push("relay status row has no local live graph cause audit".to_string());
    }
}

pub(super) fn push_state_check(
    checks: &mut Vec<Value>,
    surface_state: &Value,
    handle: Option<&str>,
    why: Option<&Value>,
    target_checks: &TargetChecks,
) {
    if target_checks.historical_outbox_store_only() {
        push_check(
            checks,
            "state",
            "not_proven",
            "durable outbox row is historical; no live outbox graph row is materialized"
                .to_string(),
        );
    } else if target_checks.relay_status_without_graph() {
        push_check(
            checks,
            "state",
            "not_proven",
            "relay status row is materialized outside this daemon's live status graph".to_string(),
        );
    } else {
        let (status, summary) = state_check_summary(surface_state, handle, why);
        push_check(checks, "state", status, summary);
    }
}
