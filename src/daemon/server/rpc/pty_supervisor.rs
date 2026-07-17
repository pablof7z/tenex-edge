use super::super::*;

#[derive(serde::Deserialize)]
struct PtySupervisorExitParams {
    pty_id: String,
    child_success: Option<bool>,
    child_exit_code: Option<u32>,
    presentation: crate::pty::PresentationSnapshot,
    recorded_at: u64,
}

pub(in crate::daemon::server) async fn rpc_pty_supervisor_exit(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: PtySupervisorExitParams =
        serde_json::from_value(params.clone()).context("parsing pty_supervisor_exit params")?;
    tracing::info!(
        pty_id = %p.pty_id,
        child_success = ?p.child_success,
        child_exit_code = ?p.child_exit_code,
        attached_clients = p.presentation.attached_clients,
        attachment_epoch = p.presentation.attachment_epoch,
        "PTY supervisor exited"
    );
    let Some(_) = wait_for_registered_session(state, &p.pty_id).await else {
        return Ok(serde_json::json!({ "accepted": false, "ended": false }));
    };
    let ended = super::super::managed_lifecycle::supervisor_exited(
        state,
        &p.pty_id,
        p.child_success,
        p.presentation,
        p.recorded_at,
    )
    .await?;
    Ok(serde_json::json!({ "accepted": true, "ended": ended }))
}

async fn wait_for_registered_session(
    state: &Arc<DaemonState>,
    pty_id: &str,
) -> Option<crate::state::Session> {
    for _ in 0..40 {
        if let Some(session) = state
            .with_store(|store| {
                store.session_for_runtime_locator(crate::state::LOCATOR_PTY, pty_id)
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
