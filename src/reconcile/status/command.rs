use trellis_core::TransactionResult;

use super::StatusEffect;
use crate::session_state::SessionState;

/// The graph's in-plan command payload. The host stamps expiration at apply time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusCommand {
    pub pubkey: String,
    pub channels: Vec<String>,
    pub title: String,
    pub state: SessionState,
    pub host: String,
    pub slug: String,
    pub rel_cwd: String,
    pub dispatch_event: Option<String>,
}

/// One reconciler transaction's effects and raw receipt.
pub struct StatusOutcome {
    pub effects: Vec<StatusEffect>,
    pub result: TransactionResult<StatusCommand>,
}
