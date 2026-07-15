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

/// Custom-agent settings applied when creating a Codex root thread.
pub struct ThreadStartConfig<'a> {
    pub developer_instructions: &'a str,
    pub config: &'a toml::Table,
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

    /// `thread/start {cwd,developerInstructions?,config?}` -> result.thread.id.
    pub async fn thread_start(
        &self,
        cwd: &Path,
        custom_agent: Option<ThreadStartConfig<'_>>,
    ) -> Result<String, RpcError> {
        let v = self
            .handle
            .request_timeout(
                "thread/start",
                thread_start_params(cwd, custom_agent),
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
    custom_agent: Option<ThreadStartConfig<'_>>,
) -> serde_json::Value {
    let mut params = serde_json::json!({ "cwd": cwd.to_string_lossy() });
    if let Some(custom_agent) = custom_agent {
        let object = params
            .as_object_mut()
            .expect("thread/start params are an object");
        object.insert(
            "developerInstructions".to_string(),
            serde_json::Value::String(custom_agent.developer_instructions.to_string()),
        );
        object.insert(
            "config".to_string(),
            serde_json::to_value(custom_agent.config)
                .expect("validated custom-agent TOML serializes to JSON"),
        );
    }
    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_start_payload_carries_custom_agent_instructions_and_config() {
        let config = toml::toml! {
            model = "gpt-5.4"
            model_reasoning_effort = "high"
            [sandbox_workspace_write]
            network_access = true
        };

        let payload = thread_start_params(
            Path::new("/workspace"),
            Some(ThreadStartConfig {
                developer_instructions: "Review like an owner",
                config: &config,
            }),
        );

        assert_eq!(
            payload,
            serde_json::json!({
                "cwd": "/workspace",
                "developerInstructions": "Review like an owner",
                "config": {
                    "model": "gpt-5.4",
                    "model_reasoning_effort": "high",
                    "sandbox_workspace_write": { "network_access": true }
                }
            })
        );
    }

    #[test]
    fn thread_start_payload_without_custom_agent_stays_cwd_only() {
        assert_eq!(
            thread_start_params(Path::new("/workspace"), None),
            serde_json::json!({ "cwd": "/workspace" })
        );
    }
}
