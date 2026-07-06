use super::super::report::str_at;
use serde_json::{json, Value};

pub(super) fn latest_receipt(explanation: &Value) -> Option<Value> {
    let receipt = explanation
        .get("receipts")
        .and_then(Value::as_array)?
        .first()?;
    let changed = serde_json::from_str::<Value>(str_at(receipt, "changed_summary")).ok();
    Some(json!({
        "id": receipt.get("id").and_then(Value::as_i64),
        "transaction_id": receipt.get("transaction_id").and_then(Value::as_i64),
        "revision": receipt.get("revision").and_then(Value::as_i64),
        "artifact_ref": receipt.get("artifact_ref").and_then(Value::as_str),
        "created_at": receipt.get("created_at").and_then(Value::as_i64),
        "kind": changed.as_ref().and_then(|v| v.get("kind")).and_then(Value::as_str),
        "shape": changed.as_ref().and_then(|v| v.get("shape")).and_then(Value::as_str),
        "frame": changed.as_ref().and_then(|v| v.get("frame")).and_then(Value::as_str),
        "emitted": changed.as_ref()
            .and_then(|v| v.pointer("/output/emitted"))
            .and_then(Value::as_bool),
        "bytes": changed.as_ref()
            .and_then(|v| v.pointer("/output/bytes"))
            .and_then(Value::as_u64),
        "input_causes": changed.as_ref()
            .and_then(|v| v.get("input_causes"))
            .cloned()
            .unwrap_or_else(|| json!([])),
    }))
}
