//! Current app-server model capabilities used as the native launch admission
//! authority. Mosaico validates the server's resolved thread configuration; it
//! does not duplicate model aliases, defaults, or effort policy.

use std::collections::HashSet;

use super::{AppServerClient, ThreadOpened, RPC_TIMEOUT};
use crate::rpc_harness::protocol::RpcErrorObject;
use crate::rpc_harness::transport::RpcError;

const PAGE_LIMIT: u32 = 100;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WirePage {
    data: Vec<ModelCapability>,
    #[serde(default)]
    next_cursor: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapability {
    model: String,
    default_reasoning_effort: String,
    supported_reasoning_efforts: Vec<ReasoningEffort>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReasoningEffort {
    reasoning_effort: String,
}

#[derive(Debug)]
pub struct ModelCatalog {
    models: Vec<ModelCapability>,
}

impl AppServerClient {
    /// Read every page of the native catalog. Cursor repetition is a protocol
    /// failure: silently accepting a partial list would advertise unsupported
    /// sessions as ready.
    pub async fn model_catalog(&self) -> Result<ModelCatalog, RpcError> {
        let mut models = Vec::new();
        let mut cursor: Option<String> = None;
        let mut seen = HashSet::new();
        loop {
            let value = self
                .handle
                .request_timeout(
                    "model/list",
                    serde_json::json!({
                        "cursor": cursor,
                        "includeHidden": true,
                        "limit": PAGE_LIMIT
                    }),
                    RPC_TIMEOUT,
                )
                .await?;
            let page: WirePage = serde_json::from_value(value).map_err(|error| {
                protocol_error(format!(
                    "model/list response does not match the current app-server schema: {error}"
                ))
            })?;
            models.extend(page.data);
            let Some(next) = page.next_cursor else {
                break;
            };
            if next.is_empty() || !seen.insert(next.clone()) {
                return Err(protocol_error(
                    "model/list returned an empty or repeated nextCursor".to_string(),
                ));
            }
            cursor = Some(next);
        }
        Ok(ModelCatalog { models })
    }
}

impl ModelCatalog {
    /// Validate the exact model and effort the native server resolved for the
    /// thread. `None` means the server selected its catalog default.
    pub fn admit(&self, opened: &ThreadOpened) -> Result<(), RpcError> {
        let matches = self
            .models
            .iter()
            .filter(|model| model.model == opened.model)
            .collect::<Vec<_>>();
        let [model] = matches.as_slice() else {
            return Err(protocol_error(format!(
                "resolved model {:?} has {} exact entries in model/list; refusing readiness",
                opened.model,
                matches.len()
            )));
        };
        let effort = opened
            .reasoning_effort
            .as_deref()
            .unwrap_or(&model.default_reasoning_effort);
        if !model
            .supported_reasoning_efforts
            .iter()
            .any(|supported| supported.reasoning_effort == effort)
        {
            return Err(protocol_error(format!(
                "resolved reasoning effort {effort:?} is unsupported for model {:?}",
                opened.model
            )));
        }
        Ok(())
    }
}

pub(super) fn protocol_error(message: String) -> RpcError {
    RpcError::Protocol(RpcErrorObject {
        code: -1,
        message,
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc_harness::{Callbacks, Dialect, RpcHandle, SpawnConfig};

    fn catalog() -> ModelCatalog {
        ModelCatalog {
            models: vec![ModelCapability {
                model: "gpt-current".into(),
                default_reasoning_effort: "medium".into(),
                supported_reasoning_efforts: vec![
                    ReasoningEffort {
                        reasoning_effort: "low".into(),
                    },
                    ReasoningEffort {
                        reasoning_effort: "medium".into(),
                    },
                ],
            }],
        }
    }

    #[test]
    fn exact_native_model_and_effort_are_required() {
        let opened = |model: &str, effort: Option<&str>| ThreadOpened {
            thread_id: "thread".into(),
            model: model.into(),
            reasoning_effort: effort.map(str::to_string),
        };
        assert!(catalog().admit(&opened("gpt-current", Some("low"))).is_ok());
        assert!(catalog().admit(&opened("gpt-current", None)).is_ok());
        assert!(catalog().admit(&opened("gpt-alias", Some("low"))).is_err());
        assert!(catalog()
            .admit(&opened("gpt-current", Some("ultra")))
            .is_err());
    }

    #[tokio::test]
    async fn every_native_catalog_page_is_required() {
        let cwd = std::env::temp_dir();
        let script = r#"
IFS= read -r first || exit 1
printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"data":[{"model":"first","defaultReasoningEffort":"low","supportedReasoningEfforts":[{"reasoningEffort":"low"}]}],"nextCursor":"page-2"}}'
IFS= read -r second || exit 1
case "$second" in *'"cursor":"page-2"'*) ;; *) exit 2 ;; esac
printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"data":[{"model":"second","defaultReasoningEffort":"high","supportedReasoningEfforts":[{"reasoningEffort":"high"}]}],"nextCursor":null}}'
while IFS= read -r line; do :; done
"#;
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
        let catalog = AppServerClient::new(handle.clone())
            .model_catalog()
            .await
            .unwrap();
        assert!(catalog
            .admit(&ThreadOpened {
                thread_id: "thread".into(),
                model: "second".into(),
                reasoning_effort: Some("high".into()),
            })
            .is_ok());
        handle.kill().await.unwrap();
    }
}
