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
    let catalog = client
        .model_catalog()
        .await
        .map_err(|error| anyhow::anyhow!("app-server model/list: {error}"))?;
    let agent = resolve(activation)?;
    let opened = client
        .thread_start(cwd, agent.developer_instructions, agent.config.as_ref())
        .await
        .map_err(|error| anyhow::anyhow!("app-server thread/start: {error}"))?;
    catalog
        .admit(&opened)
        .map_err(|error| anyhow::anyhow!("app-server launch admission: {error}"))?;
    Ok(opened.thread_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc_harness::{Callbacks, Dialect, SpawnConfig};

    fn fixture(resolved_model: &str, resolved_effort: &str) -> SpawnConfig {
        let script = format!(
            r#"
IFS= read -r initialize || exit 1
printf '%s\n' '{{"jsonrpc":"2.0","id":1,"result":{{}}}}'
IFS= read -r models || exit 1
printf '%s\n' '{{"jsonrpc":"2.0","id":2,"result":{{"data":[{{"model":"gpt-supported","defaultReasoningEffort":"medium","supportedReasoningEfforts":[{{"reasoningEffort":"medium"}}]}}],"nextCursor":null}}}}'
IFS= read -r start || exit 1
printf '%s\n' '{{"jsonrpc":"2.0","id":3,"result":{{"thread":{{"id":"thread-1","turns":[]}},"model":"{resolved_model}","reasoningEffort":"{resolved_effort}"}}}}'
while IFS= read -r line; do :; done
"#
        );
        let cwd = std::env::temp_dir();
        SpawnConfig {
            program: "/bin/sh".into(),
            args: vec!["-c".into(), script],
            cwd: cwd.clone(),
            env: Vec::new(),
            env_remove: Vec::new(),
            dialect: Dialect::AppServer,
            callbacks: Callbacks::allow_all(cwd),
        }
    }

    #[tokio::test]
    async fn native_catalog_admits_only_the_exact_resolved_thread() {
        let (supported, _) = RpcHandle::spawn(fixture("gpt-supported", "medium"))
            .await
            .unwrap();
        assert_eq!(
            open(&supported, Path::new("/workspace"), None)
                .await
                .unwrap(),
            "thread-1"
        );
        supported.kill().await.unwrap();

        let (unsupported, _) = RpcHandle::spawn(fixture("gpt-missing", "medium"))
            .await
            .unwrap();
        let error = open(&unsupported, Path::new("/workspace"), None)
            .await
            .unwrap_err();
        assert!(format!("{error:#}").contains("0 exact entries in model/list"));
        unsupported.kill().await.unwrap();
    }

    #[tokio::test]
    async fn unavailable_native_catalog_rejects_before_thread_start() {
        let script = r#"
IFS= read -r initialize || exit 1
printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{}}'
IFS= read -r models || exit 1
printf '%s\n' '{"jsonrpc":"2.0","id":2,"error":{"code":-32601,"message":"model/list unavailable"}}'
IFS= read -r unexpected && exit 2
"#;
        let cwd = std::env::temp_dir();
        let (handle, _) = RpcHandle::spawn(SpawnConfig {
            program: "/bin/sh".into(),
            args: vec!["-c".into(), script.into()],
            cwd: cwd.clone(),
            env: Vec::new(),
            env_remove: Vec::new(),
            dialect: Dialect::AppServer,
            callbacks: Callbacks::allow_all(cwd),
        })
        .await
        .unwrap();

        let error = open(&handle, Path::new("/workspace"), None)
            .await
            .unwrap_err();

        assert!(format!("{error:#}").contains("model/list unavailable"));
        handle.kill().await.unwrap();
    }
}
