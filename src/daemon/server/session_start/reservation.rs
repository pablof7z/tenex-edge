use super::*;

pub(in crate::daemon::server) async fn rpc_session_start(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
    progress: Option<InitProgress>,
) -> Result<serde_json::Value> {
    rpc_session_start_inner(state, params, progress).await
}
