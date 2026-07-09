use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, Stdout};
use tokio::sync::Mutex;

pub(super) type SharedWriter = Arc<Mutex<Stdout>>;

pub(super) const PARSE_ERROR: i64 = -32700;
pub(super) const INVALID_REQUEST: i64 = -32600;
pub(super) const METHOD_NOT_FOUND: i64 = -32601;
pub(super) const INVALID_PARAMS: i64 = -32602;

#[derive(Debug, Deserialize)]
pub(super) struct Message {
    #[serde(default)]
    pub(super) id: Option<Value>,
    #[serde(default)]
    pub(super) method: Option<String>,
    #[serde(default)]
    pub(super) params: Value,
}

impl Message {
    pub(super) fn parse(line: &str) -> Result<Self> {
        serde_json::from_str(line).context("parsing MCP JSON-RPC message")
    }

    pub(super) fn is_notification(&self) -> bool {
        self.method.is_some() && self.id.is_none()
    }
}

pub(super) async fn write_value(writer: &SharedWriter, value: &Value) -> Result<()> {
    let mut line = serde_json::to_string(value)?;
    line.push('\n');
    let mut out = writer.lock().await;
    out.write_all(line.as_bytes()).await?;
    out.flush().await?;
    Ok(())
}

pub(super) fn result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

pub(super) fn error(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message.into(),
        },
    })
}

pub(super) fn notification(method: &str, params: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    })
}

pub(super) fn required_string(params: &Value, key: &str) -> Result<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(ToString::to_string)
        .with_context(|| format!("missing required string parameter {key:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_request_and_notification() {
        let req = Message::parse(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#).unwrap();
        assert_eq!(req.method.as_deref(), Some("ping"));
        assert!(!req.is_notification());

        let note =
            Message::parse(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).unwrap();
        assert!(note.is_notification());
    }
}
