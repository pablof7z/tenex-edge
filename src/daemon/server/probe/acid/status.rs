use std::sync::Arc;

use anyhow::{Context, Result};

use super::DaemonState;

pub(super) fn current_arm_at(state: &Arc<DaemonState>, session_id: &str) -> Result<u64> {
    state
        .status
        .lock()
        .expect("status mutex poisoned")
        .current_arm_at(session_id)
        .with_context(|| format!("probe acid: no status arm for `{session_id}`"))
}
