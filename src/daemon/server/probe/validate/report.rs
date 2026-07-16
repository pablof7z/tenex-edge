//! Reporting helpers for `probe validate`.

use serde_json::{json, Value};

mod seams;
pub(super) use seams::{seams_status, seams_summary};

pub(super) fn push_check(checks: &mut Vec<Value>, name: &str, status: &str, summary: String) {
    checks.push(json!({ "name": name, "status": status, "summary": summary }));
}

pub(super) fn verdict(checks: &[Value], limitations: &[String]) -> &'static str {
    if checks.iter().any(|c| str_at(c, "status") == "failed") {
        "failed"
    } else if !limitations.is_empty() || checks.iter().any(|c| str_at(c, "status") == "not_proven")
    {
        "passed_with_limitations"
    } else {
        "passed"
    }
}

pub(super) fn drift_surfaces(stats: &Value) -> Vec<String> {
    stats
        .get("surfaces")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| bool_at(row, "resource_drift"))
        .map(|row| str_at(row, "surface").to_string())
        .collect()
}

pub(super) fn explain_found(v: &Value) -> bool {
    has_receipts(v)
}

pub(super) fn explain_summary(v: &Value) -> String {
    let receipts = v
        .get("receipts")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    format!("{} explanation: {receipts} receipt(s)", str_at(v, "kind"))
}

pub(super) fn receipt_surface(v: &Value) -> Option<String> {
    let mut surfaces = v
        .get("receipts")
        .and_then(Value::as_array)?
        .iter()
        .filter_map(|row| row.get("surface").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    surfaces.sort();
    surfaces.dedup();
    match surfaces.as_slice() {
        [surface] => Some(surface.clone()),
        _ => None,
    }
}

pub(super) fn chosen_cause(
    explicit: Option<&str>,
    simulation: &Value,
    why: Option<&Value>,
) -> Option<String> {
    explicit.map(str::to_string).or_else(|| {
        let changed = labels_at(simulation, "changed");
        let why_causes = why.map_or_else(Vec::new, |v| labels_at(v, "input_causes"));
        if first_command_op(simulation).as_deref() == Some("Refresh") {
            changed
                .iter()
                .chain(why_causes.iter())
                .find(|label| label.ends_with("/arm"))
                .cloned()
                .or_else(|| changed.first().cloned())
                .or_else(|| why_causes.first().cloned())
        } else {
            changed
                .iter()
                .chain(why_causes.iter())
                .find(|label| !label.ends_with("/arm"))
                .cloned()
                .or_else(|| changed.first().cloned())
                .or_else(|| why_causes.first().cloned())
        }
    })
}

fn first_command_op(v: &Value) -> Option<String> {
    v.get("commands")?
        .as_array()?
        .first()?
        .get("op")?
        .as_str()
        .map(str::to_string)
}

pub(super) fn oracle_summary(v: &Value) -> String {
    if bool_at(v, "ok") {
        "all live Trellis graph oracles are green".to_string()
    } else {
        format!("red surfaces: {}", red_surfaces(v).join(", "))
    }
}

fn red_surfaces(v: &Value) -> Vec<String> {
    v.get("surfaces")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| str_at(row, "status") == "red")
        .map(|row| str_at(row, "surface").to_string())
        .collect()
}

pub(super) fn why_summary(v: &Value) -> String {
    if !bool_at(v, "found") {
        return str_at(v, "note").to_string();
    }
    format!(
        "latest {} command {} caused by {}",
        str_at(v, "kind"),
        str_at(v, "last_kind"),
        labels_at(v, "input_causes").join(", ")
    )
}

pub(super) fn simulate_summary(v: &Value) -> String {
    if v.get("simulated").and_then(Value::as_bool) == Some(false) {
        return v
            .get("fact_evidence")
            .and_then(|fact| fact.get("summary"))
            .and_then(Value::as_str)
            .unwrap_or("fact has no validating Trellis surface yet")
            .to_string();
    }
    let commands = v
        .get("commands")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if bool_at(v, "would_publish") {
        let Some(first) = commands.first() else {
            return "would publish, but no command details were returned".into();
        };
        format!(
            "would publish kind:{} {} {} without mutating live state",
            int_at(first, "kind"),
            str_at(first, "op"),
            str_at(first, "resource")
        )
    } else if commands.is_empty() && int_at(v, "output_frames") > 0 {
        format!(
            "would emit {} output frame(s) without mutating live state",
            int_at(v, "output_frames")
        )
    } else if bool_at(v, "would_effect") {
        format!(
            "would apply {} command(s) without mutating live state",
            commands.len()
        )
    } else {
        "would make no change".to_string()
    }
}

pub(super) fn acid_summary(v: &Value) -> String {
    format!(
        "cause {} necessary={} unrelated_stable={}",
        str_at(v, "cause"),
        bool_at(v, "necessary"),
        bool_at(v, "unrelated_stable")
    )
}

pub(super) fn replay_summary(v: &Value) -> String {
    format!(
        "capsule {} deterministic replay ok; steps={} commands={}",
        int_at(v.get("capsule").unwrap_or(&Value::Null), "id"),
        int_at(v, "steps"),
        int_at(v, "resource_commands")
    )
}

fn has_receipts(v: &Value) -> bool {
    v.get("receipts")
        .and_then(Value::as_array)
        .is_some_and(|rows| !rows.is_empty())
}

fn labels_at(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

pub(super) fn str_at<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

pub(super) fn bool_at(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

pub(super) fn int_at(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}
