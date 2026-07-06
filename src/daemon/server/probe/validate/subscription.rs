//! Subscription target validation.

use super::report::{bool_at, int_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) struct SubscriptionTarget {
    kind: &'static str,
    entity: String,
    resources: Vec<String>,
}

pub(super) fn subscription_target(target: &str) -> Option<SubscriptionTarget> {
    if let Some(channel) = target.strip_prefix("sub:").filter(|s| !s.is_empty()) {
        return Some(SubscriptionTarget {
            kind: "channel",
            entity: channel.to_string(),
            resources: vec![format!("sub/h/{channel}"), format!("sub/d/{channel}")],
        });
    }
    let rest = target.strip_prefix("sub/")?;
    let (space, entity) = rest.split_once('/')?;
    if !matches!(space, "h" | "d" | "p") || entity.is_empty() {
        return None;
    }
    let resource = format!("sub/{space}/{}", entity.split('/').next().unwrap_or(entity));
    Some(SubscriptionTarget {
        kind: "resource",
        entity: entity.to_string(),
        resources: vec![resource],
    })
}

pub(super) fn subscription_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    parsed: &SubscriptionTarget,
) -> Value {
    let (resources, revision) = {
        let r = state.subs.lock().expect("subs mutex poisoned");
        let rows = r.state_rows();
        let resources = parsed
            .resources
            .iter()
            .map(|resource| {
                let row = rows.iter().find(|row| row.resource_key == *resource);
                let why = r.explain_resource_path(resource);
                json!({
                    "resource_key": resource,
                    "found": row.is_some(),
                    "refcount": row.map(|row| row.refcount).unwrap_or(0),
                    "owners": row.map(|row| row.owners.clone()).unwrap_or_default(),
                    "last_kind": why.as_ref().and_then(|why| why.last_kind.as_deref()).unwrap_or(""),
                    "cause": why.as_ref().and_then(|why| why.cause.as_deref()).unwrap_or(""),
                    "input_causes": why.map(|why| why.input_causes).unwrap_or_default(),
                })
            })
            .collect::<Vec<_>>();
        (resources, r.revision())
    };
    let found_count = resources
        .iter()
        .filter(|resource| bool_at(resource, "found"))
        .count();
    let receipt_count = subscription_receipt_count(state, &parsed.entity).unwrap_or(0);
    let ok = found_count == parsed.resources.len();

    json!({
        "target": target,
        "kind": parsed.kind,
        "entity": parsed.entity,
        "supported": true,
        "found": found_count > 0,
        "ok": ok,
        "revision": revision,
        "expected_resource_count": parsed.resources.len(),
        "found_resource_count": found_count,
        "receipt_count": receipt_count,
        "resources": resources,
        "summary": summary(&parsed.entity, parsed.kind, found_count, parsed.resources.len()),
        "reason": reason(parsed.kind, found_count, parsed.resources.len()),
    })
}

pub(super) fn push_subscription_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let status = if bool_at(evidence, "ok") {
        "passed"
    } else if int_at(evidence, "found_resource_count") > 0 {
        "failed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "subscription_outcome",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn subscription_receipt_count(state: &Arc<DaemonState>, entity: &str) -> Option<usize> {
    state
        .with_store(|s| {
            crate::explain::explain(
                s,
                &crate::explain::Handle::Sub {
                    channel: entity.to_string(),
                },
            )
        })
        .ok()
        .and_then(|v| v.get("receipts").and_then(Value::as_array).map(Vec::len))
}

fn summary(entity: &str, kind: &str, found: usize, expected: usize) -> String {
    if found == expected {
        format!("subscription `{entity}` has all {expected} expected {kind} resource(s)")
    } else if found > 0 {
        format!("subscription `{entity}` is partially materialized ({found}/{expected})")
    } else {
        format!("subscription `{entity}` has no live resource evidence")
    }
}

fn reason(kind: &str, found: usize, expected: usize) -> &'static str {
    if found == expected {
        ""
    } else if found > 0 && kind == "channel" {
        "channel subscriptions must materialize both sub/h and sub/d resources"
    } else if found > 0 {
        "requested subscription resource is only partially materialized"
    } else {
        "requested subscription resource is not materialized in the live graph"
    }
}
