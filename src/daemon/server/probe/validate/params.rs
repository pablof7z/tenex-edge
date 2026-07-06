//! Parameter-shape and input helpers for `probe validate`.

use super::super::artifact;
use crate::reconcile::journal::InputFact;
use serde_json::{json, Value};

pub(super) fn malformed_parameter_evidence(params: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    expect_string(params, "target", &mut out);
    expect_string(params, "capsule", &mut out);
    expect_string(params, "cause", &mut out);
    expect_integer_string(params, "capsule", "replay capsule id", &mut out);
    if params
        .get("since")
        .is_some_and(|v| !v.is_null() && !v.is_i64())
    {
        out.push(json!({
            "parameter": "since",
            "kind": "invalid_parameter",
            "valid": false,
            "summary": "parameter `since` must be an integer unix-millis stamp",
            "reason": "validate parameter `since` must be an integer unix-millis stamp",
        }));
    }
    out
}

pub(super) fn has_invalid_parameter(evidence: &[Value], parameter: &str) -> bool {
    evidence.iter().any(|v| {
        v.get("parameter").and_then(Value::as_str) == Some(parameter)
            && v.get("valid").and_then(Value::as_bool) == Some(false)
    })
}

pub(super) fn simulate_params(params: &Value, surface: Option<&str>) -> Value {
    let fact = params
        .get("fact")
        .cloned()
        .filter(|v| !v.is_null())
        .unwrap_or(Value::Null);
    let mut out = json!({ "verb": "simulate", "fact": fact });
    if let Some(surface) = surface {
        out["surface"] = Value::String(surface.to_string());
    }
    out
}

pub(super) fn fact_param_for_validation(params: &Value) -> (Option<InputFact>, Option<Value>) {
    match artifact::fact_param(params, "fact") {
        Ok(fact) => (fact, None),
        Err(e) if has_value(params, "fact") => (
            None,
            Some(json!({
                "kind": "InvalidInputFact",
                "supported": false,
                "valid": false,
                "frontier": "input_decode",
                "summary": "fact is not a valid InputFact",
                "reason": format!("{e:#}"),
            })),
        ),
        Err(_) => (None, None),
    }
}

pub(super) fn has_value(params: &Value, key: &str) -> bool {
    params.get(key).is_some_and(|v| !v.is_null())
}

fn expect_string(params: &Value, key: &str, out: &mut Vec<Value>) {
    let Some(value) = params.get(key).filter(|v| !v.is_null()) else {
        return;
    };
    match value.as_str() {
        None => {
            out.push(json!({
                "parameter": key,
                "kind": "invalid_parameter",
                "valid": false,
                "summary": format!("parameter `{key}` must be a string"),
                "reason": format!("validate parameter `{key}` must be a string"),
            }));
        }
        Some("") => {
            out.push(json!({
                "parameter": key,
                "kind": "invalid_parameter",
                "valid": false,
                "summary": format!("parameter `{key}` must be non-empty when provided"),
                "reason": format!("validate parameter `{key}` must be a non-empty string when provided"),
            }));
        }
        Some(_) => {}
    }
}

fn expect_integer_string(params: &Value, key: &str, label: &str, out: &mut Vec<Value>) {
    let Some(value) = params
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    if value.parse::<i64>().is_ok() {
        return;
    }
    out.push(json!({
        "parameter": key,
        "kind": "invalid_parameter",
        "valid": false,
        "summary": format!("parameter `{key}` must be an integer {label}"),
        "reason": format!("validate parameter `{key}` must be an integer {label}"),
    }));
}
