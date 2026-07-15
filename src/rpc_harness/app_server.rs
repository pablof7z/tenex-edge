//! Codex `app-server` dialect wrappers over [`RpcHandle`].
//!
//! Key asymmetry vs ACP: `turn/start` does NOT resolve on its JSON-RPC
//! response; the turn completes via a separate `turn/completed` notification.
//! So `turn_start` registers a completion waiter keyed by `threadId` before
//! sending, then awaits that waiter. `turn/steer` DOES resolve on its response.

use std::path::Path;
use std::time::Duration;

use super::protocol::RpcErrorObject;
use super::transport::{RpcError, RpcHandle};

const RPC_TIMEOUT: Duration = Duration::from_secs(60);
const TURN_TIMEOUT: Duration = Duration::from_secs(300);

pub struct AppServerClient {
    pub handle: RpcHandle,
}

/// Outcome of a completed app-server turn.
#[derive(Debug, Clone)]
pub struct TurnOutcome {
    pub raw: serde_json::Value,
}

impl AppServerClient {
    pub fn new(handle: RpcHandle) -> Self {
        Self { handle }
    }

    /// `initialize {clientInfo:{name,title,version}}`.
    pub async fn initialize(
        &self,
        name: &str,
        version: &str,
    ) -> Result<serde_json::Value, RpcError> {
        self.handle
            .request_timeout(
                "initialize",
                serde_json::json!({
                    "clientInfo": { "name": name, "title": name, "version": version }
                }),
                RPC_TIMEOUT,
            )
            .await
    }

    /// `config/read {cwd,includeLayers:true}` -> effective process config.
    pub async fn config_read(&self, cwd: &Path) -> Result<serde_json::Value, RpcError> {
        self.handle
            .request_timeout(
                "config/read",
                serde_json::json!({
                    "cwd": cwd.to_string_lossy(),
                    "includeLayers": true
                }),
                RPC_TIMEOUT,
            )
            .await
    }

    /// Start a thread, optionally applying a resolved Codex custom agent at the
    /// app-server's native instruction/config boundary.
    pub async fn thread_start(
        &self,
        cwd: &Path,
        developer_instructions: Option<&str>,
        config: Option<&serde_json::Value>,
    ) -> Result<String, RpcError> {
        let v = self
            .handle
            .request_timeout(
                "thread/start",
                thread_start_params(cwd, developer_instructions, config),
                RPC_TIMEOUT,
            )
            .await?;
        v.get("thread")
            .and_then(|t| t.get("id"))
            .and_then(|i| i.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                RpcError::Protocol(RpcErrorObject {
                    code: -1,
                    message: format!("thread/start missing thread.id: {v}"),
                    data: None,
                })
            })
    }

    /// `turn/start {threadId, input:[{type:text,text}]}`, resolved by the
    /// `turn/completed` notification.
    pub async fn turn_start(&self, thread_id: &str, text: &str) -> Result<TurnOutcome, RpcError> {
        let waiter = self.handle.register_turn_waiter(thread_id);
        // Send turn/start; its immediate response is not the completion.
        self.handle
            .request_timeout(
                "turn/start",
                serde_json::json!({
                    "threadId": thread_id,
                    "input": [{ "type": "text", "text": text }]
                }),
                RPC_TIMEOUT,
            )
            .await?;
        match tokio::time::timeout(TURN_TIMEOUT, waiter).await {
            Ok(Ok(raw)) => Ok(TurnOutcome { raw }),
            Ok(Err(_)) => Err(RpcError::ChildExited),
            Err(_) => Err(RpcError::Timeout),
        }
    }

    /// `turn/steer {threadId, expectedTurnId, input}` — mid-turn inject; resolves
    /// on its own response.
    pub async fn turn_steer(
        &self,
        thread_id: &str,
        expected_turn_id: &str,
        text: &str,
    ) -> Result<(), RpcError> {
        self.handle
            .request_timeout(
                "turn/steer",
                serde_json::json!({
                    "threadId": thread_id,
                    "expectedTurnId": expected_turn_id,
                    "input": [{ "type": "text", "text": text }]
                }),
                RPC_TIMEOUT,
            )
            .await
            .map(|_| ())
    }

    /// `thread/resume {threadId, cwd}`.
    pub async fn thread_resume(&self, thread_id: &str, cwd: &Path) -> Result<(), RpcError> {
        self.handle
            .request_timeout(
                "thread/resume",
                serde_json::json!({
                    "threadId": thread_id,
                    "cwd": cwd.to_string_lossy()
                }),
                RPC_TIMEOUT,
            )
            .await
            .map(|_| ())
    }
}

fn thread_start_params(
    cwd: &Path,
    developer_instructions: Option<&str>,
    config: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut params = serde_json::json!({ "cwd": cwd.to_string_lossy() });
    let object = params.as_object_mut().expect("thread/start params object");
    if let Some(instructions) = developer_instructions {
        object.insert(
            "developerInstructions".to_string(),
            serde_json::Value::String(instructions.to_string()),
        );
    }
    if let Some(config) = config {
        object.insert("config".to_string(), config.clone());
    }
    params
}

#[cfg(test)]
mod tests {
    use super::thread_start_params;

    #[test]
    fn custom_agent_uses_native_thread_start_fields() {
        let cwd = std::path::Path::new("/workspace");
        let config = serde_json::json!({
            "model": "gpt-5.4",
            "model_reasoning_effort": "high"
        });

        assert_eq!(
            thread_start_params(cwd, Some("Review carefully"), Some(&config)),
            serde_json::json!({
                "cwd": "/workspace",
                "developerInstructions": "Review carefully",
                "config": {
                    "model": "gpt-5.4",
                    "model_reasoning_effort": "high"
                }
            })
        );
    }

    #[test]
    fn default_thread_start_omits_agent_fields() {
        assert_eq!(
            thread_start_params(std::path::Path::new("/workspace"), None, None),
            serde_json::json!({ "cwd": "/workspace" })
        );
    }
}
