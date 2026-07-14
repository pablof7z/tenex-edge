use super::super::*;

#[derive(serde::Deserialize)]
struct PtySupervisorExitParams {
    pty_id: String,
    #[serde(default)]
    durable_reservation: Option<String>,
}

pub(in crate::daemon::server) async fn rpc_pty_supervisor_exit(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: PtySupervisorExitParams =
        serde_json::from_value(params.clone()).context("parsing pty_supervisor_exit params")?;
    let session = wait_for_registered_session(state, &p.pty_id).await;
    if let Some(session) = session {
        return rpc_session_end(state, &serde_json::json!({ "session": session.session_id })).await;
    }
    if let Some(reservation) = p.durable_reservation.as_deref() {
        state.with_store(|store| store.release_durable_agent_session(reservation))?;
    }
    Ok(serde_json::json!({ "ended": false }))
}

async fn wait_for_registered_session(
    state: &Arc<DaemonState>,
    pty_id: &str,
) -> Option<crate::state::Session> {
    for _ in 0..40 {
        if let Some(session) = state.with_store(|store| store.get_session(pty_id).ok().flatten()) {
            return Some(session);
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    None
}
