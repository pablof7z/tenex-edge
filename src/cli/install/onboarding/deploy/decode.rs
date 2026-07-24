//! Decode raw RPC `SessionUpdate` notifications into normalized [`DeployEvent`]s.
//!
//! Only the agent-message-chunk shape is exercised by existing repo fixtures
//! (`acp_runtime_tests`), so that path is decoded precisely. Tool/command and
//! app-server item shapes are not verified against a live harness here, so they
//! render as generic one-line activity rather than guessing field layouts.

use crate::rpc_harness::SessionUpdate;
use serde_json::Value;

use super::transcript::DeployEvent;

/// Turn/lifecycle notifications are handled by the driver's turn future, not the
/// transcript stream.
const LIFECYCLE: &[&str] = &["turn/completed", "thread/status/changed"];

pub(in crate::cli::install::onboarding) fn decode(update: &SessionUpdate) -> Option<DeployEvent> {
    if LIFECYCLE.contains(&update.method.as_str()) {
        return None;
    }
    // ACP `session/update` carries the payload under `params.update`.
    if let Some(inner) = update.params.get("update") {
        return decode_acp_update(inner);
    }
    // App-server item notifications: summarize generically.
    decode_app_server(update)
}

fn decode_acp_update(update: &Value) -> Option<DeployEvent> {
    let kind = update.get("sessionUpdate").and_then(Value::as_str)?;
    match kind {
        "agent_message_chunk" => Some(DeployEvent::Agent(content_text(update)?)),
        "agent_thought_chunk" => Some(DeployEvent::Thought(content_text(update)?)),
        "tool_call" | "tool_call_update" => Some(DeployEvent::Activity(tool_summary(update))),
        "plan" => Some(DeployEvent::Activity("planning…".into())),
        _ => None,
    }
}

/// Extract `content.text` from an ACP chunk (`content` may be an object or an
/// array of content blocks).
fn content_text(update: &Value) -> Option<String> {
    let content = update.get("content")?;
    if let Some(text) = content.get("text").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    if let Some(blocks) = content.as_array() {
        let joined: String = blocks
            .iter()
            .filter_map(|b| b.get("text").and_then(Value::as_str))
            .collect();
        if !joined.is_empty() {
            return Some(joined);
        }
    }
    None
}

fn tool_summary(update: &Value) -> String {
    let title = update
        .get("title")
        .or_else(|| update.get("kind"))
        .or_else(|| update.get("toolName"))
        .and_then(Value::as_str)
        .unwrap_or("tool");
    let status = update
        .get("status")
        .and_then(Value::as_str)
        .map(|s| format!(" [{s}]"))
        .unwrap_or_default();
    format!("{title}{status}")
}

fn decode_app_server(update: &SessionUpdate) -> Option<DeployEvent> {
    // Only surface item-related notifications; skip bookkeeping methods.
    if !update.method.starts_with("item/") && !update.method.starts_with("codex/") {
        return None;
    }
    // Best-effort: prefer any human-meaningful text field, else the method name.
    let text = first_str(&update.params, &["text", "message", "title", "command"]);
    Some(DeployEvent::Activity(match text {
        Some(t) => t,
        None => update.method.clone(),
    }))
}

fn first_str(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(found) = deep_find_str(value, key) {
            return Some(found);
        }
    }
    None
}

/// Shallow-then-nested search for the first string value under `key`.
fn deep_find_str(value: &Value, key: &str) -> Option<String> {
    match value {
        Value::Object(map) => {
            if let Some(s) = map.get(key).and_then(Value::as_str) {
                return Some(s.to_string());
            }
            map.values().find_map(|v| deep_find_str(v, key))
        }
        Value::Array(items) => items.iter().find_map(|v| deep_find_str(v, key)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "decode_tests.rs"]
mod tests;
