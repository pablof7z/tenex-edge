use super::*;

pub(super) async fn rpc_sessions(state: &Arc<DaemonState>) {
    for (kind, endpoint, confirmation) in
        crate::session_host::transport::acp::shutdown_owned_sessions().await
    {
        match confirmation {
            Ok(()) => {
                let session = state
                    .with_store(|store| {
                        store.session_for_runtime_locator(kind.locator_kind(), &endpoint)
                    })
                    .ok()
                    .flatten();
                if let Some(session) = session {
                    if let Err(error) = state.with_store(|store| {
                        store.mark_runtime_stopped_if_generation(
                            &session.pubkey,
                            session.runtime_generation,
                            crate::state::StopReason::Superseded,
                            crate::util::now_secs(),
                        )
                    }) {
                        tracing::error!(%endpoint, %error, "RPC shutdown state update failed");
                    }
                }
            }
            Err(error) => {
                tracing::error!(%endpoint, %error, "RPC process-group shutdown failed");
            }
        }
    }
}
