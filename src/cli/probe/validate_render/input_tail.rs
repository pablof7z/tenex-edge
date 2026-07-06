//! Generic input/target/fact evidence renderers for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render_parameters(out: &mut String, params: &[Value]) {
    if params.is_empty() {
        return;
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "parameter evidence");
    for param in params {
        let _ = writeln!(out, "  - {}: invalid", str_at(param, "parameter"));
        if !str_at(param, "reason").is_empty() {
            let _ = writeln!(out, "    {}", str_at(param, "reason"));
        }
    }
}

pub(super) fn render_target(out: &mut String, target: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "target evidence");
    let status = if target.get("valid").and_then(Value::as_bool) == Some(false) {
        "invalid"
    } else if bool_at(target, "supported") {
        "covered"
    } else {
        "not proven"
    };
    let target_name = str_at(target, "target");
    let separator = if target_name.ends_with(':') {
        " "
    } else {
        ": "
    };
    let _ = writeln!(out, "  - {target_name}{separator}{status}");
    if !str_at(target, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(target, "reason"));
    }
}

pub(super) fn render_cause_label(out: &mut String, cause: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "cause label evidence");
    let _ = writeln!(
        out,
        "  - {}: {}",
        str_at(cause, "label"),
        str_at(cause, "surface")
    );
    if !str_at(cause, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(cause, "reason"));
    }
}

pub(super) fn render_fact(out: &mut String, fact: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "fact evidence");
    let status = if bool_at(fact, "supported") {
        "covered"
    } else if fact.get("valid").and_then(Value::as_bool) == Some(false) {
        "invalid"
    } else {
        "not proven"
    };
    let surface = str_at(fact, "surface");
    if surface.is_empty() {
        let _ = writeln!(
            out,
            "  - {}: {} ({})",
            str_at(fact, "kind"),
            status,
            str_at(fact, "frontier")
        );
    } else {
        let _ = writeln!(
            out,
            "  - {}: {} by {}",
            str_at(fact, "kind"),
            status,
            surface
        );
    }
    if !str_at(fact, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(fact, "reason"));
    }
}

fn str_at<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn bool_at(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}
