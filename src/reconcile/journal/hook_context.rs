use serde::{Deserialize, Serialize};

/// Frozen hook-context render input. JSON keeps the private render-capture
/// structure out of the public fact API.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookContextRenderFact {
    pub pubkey: String,
    pub hook_kind: String,
    pub cursor: i64,
    pub now: i64,
    pub force: bool,
    pub emitted_text_hash: Option<String>,
    pub inputs_json: serde_json::Value,
}
