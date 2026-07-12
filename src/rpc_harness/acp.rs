//! ACP dialect method wrappers over [`RpcHandle`].
//!
//! ACP semantics: `session/prompt` resolves on `{stopReason}` (the response IS
//! turn completion), unlike app-server which signals completion via a separate
//! notification.

use std::path::Path;
use std::time::Duration;

use super::protocol::StopReason;
use super::transport::{RpcError, RpcHandle};

const PROMPT_TIMEOUT: Duration = Duration::from_secs(300);
const RPC_TIMEOUT: Duration = Duration::from_secs(60);

/// ACP protocol methods over a live handle.
pub struct AcpClient {
    pub handle: RpcHandle,
}

impl AcpClient {
    pub fn new(handle: RpcHandle) -> Self {
        Self { handle }
    }

    /// `initialize {protocolVersion:1, clientCapabilities:{fs:{...}}}`.
    pub async fn initialize(&self) -> Result<serde_json::Value, RpcError> {
        self.handle
            .request_timeout(
                "initialize",
                serde_json::json!({
                    "protocolVersion": 1,
                    "clientCapabilities": {
                        "fs": { "readTextFile": true, "writeTextFile": true }
                    }
                }),
                RPC_TIMEOUT,
            )
            .await
    }

    /// `session/new {cwd, mcpServers:[]}` -> sessionId.
    pub async fn session_new(&self, cwd: &Path) -> Result<String, RpcError> {
        let v = self
            .handle
            .request_timeout(
                "session/new",
                serde_json::json!({
                    "cwd": cwd.to_string_lossy(),
                    "mcpServers": []
                }),
                RPC_TIMEOUT,
            )
            .await?;
        extract_session_id(&v)
    }

    /// `session/load {sessionId, cwd, mcpServers:[]}` — cross-process resume.
    pub async fn session_load(&self, session_id: &str, cwd: &Path) -> Result<(), RpcError> {
        self.handle
            .request_timeout(
                "session/load",
                serde_json::json!({
                    "sessionId": session_id,
                    "cwd": cwd.to_string_lossy(),
                    "mcpServers": []
                }),
                RPC_TIMEOUT,
            )
            .await
            .map(|_| ())
    }

    /// `session/prompt {sessionId, prompt:[{type:text,text}]}` -> stopReason.
    pub async fn session_prompt(
        &self,
        session_id: &str,
        text: &str,
    ) -> Result<StopReason, RpcError> {
        let v = self
            .handle
            .request_timeout(
                "session/prompt",
                serde_json::json!({
                    "sessionId": session_id,
                    "prompt": [{ "type": "text", "text": text }]
                }),
                PROMPT_TIMEOUT,
            )
            .await?;
        let reason = v
            .get("stopReason")
            .and_then(|s| s.as_str())
            .map(StopReason::from_wire)
            .unwrap_or(StopReason::Other);
        Ok(reason)
    }

    /// `session/cancel {sessionId}` — notification (fire-and-forget).
    pub async fn session_cancel(&self, session_id: &str) {
        self.handle
            .notify(
                "session/cancel",
                serde_json::json!({ "sessionId": session_id }),
            )
            .await;
    }
}

fn extract_session_id(v: &serde_json::Value) -> Result<String, RpcError> {
    v.get("sessionId")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            RpcError::Protocol(super::protocol::RpcErrorObject {
                code: -1,
                message: format!("session/new response missing sessionId: {v}"),
                data: None,
            })
        })
}
