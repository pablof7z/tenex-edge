use crate::agent_catalog::NativeAgentActivation;
use crate::rpc_harness::{AppServerClient, RpcHandle};
use anyhow::{Context, Result};
use std::path::Path;

pub(super) struct ThreadStartAgent<'a> {
    pub(super) developer_instructions: Option<&'a str>,
    pub(super) config: Option<serde_json::Value>,
}

fn resolve(activation: Option<&NativeAgentActivation>) -> Result<ThreadStartAgent<'_>> {
    let agent = match activation {
        Some(NativeAgentActivation::CodexRoot(agent)) => Some(agent),
        Some(NativeAgentActivation::NativeSelector { .. }) => {
            anyhow::bail!("app-server native activation requires a Codex custom agent")
        }
        None => None,
    };
    let config = agent
        .map(|agent| serde_json::to_value(&agent.config))
        .transpose()
        .context("serializing Codex custom-agent thread config")?;
    Ok(ThreadStartAgent {
        developer_instructions: agent.map(|agent| agent.developer_instructions.as_str()),
        config,
    })
}

pub(super) async fn open(
    handle: &RpcHandle,
    cwd: &Path,
    activation: Option<&NativeAgentActivation>,
) -> Result<String> {
    let client = AppServerClient::new(handle.clone());
    client
        .initialize("mosaico", env!("CARGO_PKG_VERSION"))
        .await
        .map_err(|error| anyhow::anyhow!("app-server initialize: {error}"))?;
    let agent = resolve(activation)?;
    client
        .thread_start(cwd, agent.developer_instructions, agent.config.as_ref())
        .await
        .map_err(|error| anyhow::anyhow!("app-server thread/start: {error}"))
}
