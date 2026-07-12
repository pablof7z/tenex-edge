use super::*;

pub(in crate::daemon::server) async fn rpc_session_start(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
    progress: Option<InitProgress>,
) -> Result<serde_json::Value> {
    let reservation = params["durable_reservation"].as_str().map(str::to_string);
    let result = rpc_session_start_inner(state, params, progress).await;
    if result.is_err() {
        if let Some(reservation) = reservation {
            state
                .with_store(|s| s.release_durable_agent_session(&reservation))
                .ok();
        }
    }
    result
}
