use crate::daemon::server::DaemonState;
use anyhow::Result;
use std::sync::Arc;

pub(super) fn reserve(state: &Arc<DaemonState>, slug: &str) -> Result<Option<String>> {
    let admission = crate::daemon::server::rpc_agent_launch_preflight(
        state,
        &serde_json::json!({ "agent": slug }),
    )?;
    Ok(admission["durable_reservation"]
        .as_str()
        .map(str::to_string))
}

pub(super) fn release(state: &Arc<DaemonState>, reservation: Option<&str>) {
    if let Some(reservation) = reservation {
        crate::daemon::server::rpc_agent_launch_release(
            state,
            &serde_json::json!({ "durable_reservation": reservation }),
        )
        .ok();
    }
}
