use super::super::*;

#[derive(serde::Deserialize)]
struct PtySupervisorExitParams {
    pty_id: String,
}

pub(in crate::daemon::server) async fn rpc_pty_supervisor_exit(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: PtySupervisorExitParams =
        serde_json::from_value(params.clone()).context("parsing pty_supervisor_exit params")?;
    let Some(session) = wait_for_registered_session(state, &p.pty_id).await else {
        return Ok(serde_json::json!({ "ended": false }));
    };
    let ended = super::super::session_end::end_runtime_generation(
        state,
        &session.pubkey,
        session.runtime_generation,
    )
    .await?;
    Ok(serde_json::json!({ "ended": ended }))
}

async fn wait_for_registered_session(
    state: &Arc<DaemonState>,
    pty_id: &str,
) -> Option<crate::state::Session> {
    for _ in 0..40 {
        if let Some(session) = state
            .with_store(|store| {
                store.alive_session_for_locator(None, crate::state::LOCATOR_PTY, pty_id)
            })
            .ok()
            .flatten()
        {
            return Some(session);
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    None
}
