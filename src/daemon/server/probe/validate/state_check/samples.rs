use super::str_at;
use serde_json::{json, Value};

pub(super) fn sample_targets(state: &Value, limit: usize) -> Vec<Value> {
    let surface = str_at(state, "surface");
    state
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| sample_target(surface, row))
        .take(limit)
        .collect()
}

fn sample_target(surface: &str, row: &Value) -> Option<Value> {
    let target = match surface {
        "status" => format!("status:{}", str_at(row, "session")),
        "subscriptions" => str_at(row, "resource_key").to_string(),
        "turn_lifecycle" => format!("turn:{}", str_at(row, "session")),
        "cursor" => format!("cursor:{}", str_at(row, "session")),
        "session_start" => format!("session_start:{}", str_at(row, "session")),
        "session_watch" => format!("session_watch:{}", str_at(row, "session")),
        "outbox" => row
            .get("local_id")
            .and_then(Value::as_i64)
            .map(|id| format!("outbox:{id}"))
            .unwrap_or_else(|| str_at(row, "resource_key").replace('/', ":")),
        "hook_context" => format!("hook:{}", str_at(row, "session")),
        _ => return None,
    };
    (!target.ends_with(':') && !target.is_empty()).then(|| {
        json!({
            "target": target,
            "resource_key": str_at(row, "resource_key"),
        })
    })
}
