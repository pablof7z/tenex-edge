use super::super::DaemonState;
use crate::state::Session;
use std::sync::Arc;

pub(super) fn render(
    state: &Arc<DaemonState>,
    roots: &[String],
    session: &Session,
    now: u64,
    host: &str,
    backend_pubkey: &str,
    all_workspaces: bool,
) -> String {
    let instance = state.session_instance(session);
    let self_name = instance.display_slug();
    let self_pubkey = instance.pubkey.clone();
    let current_root = state.with_store(|store| {
        store
            .root_channel_of(&session.channel_h)
            .ok()
            .flatten()
            .unwrap_or_else(|| session.channel_h.clone())
    });
    state.with_store(|store| {
        crate::who_view::render_agent_who(
            store,
            crate::who_view::AgentWhoInput {
                roots,
                current_root: &current_root,
                self_name: &self_name,
                self_pubkey: &self_pubkey,
                local_host: host,
                backend_pubkey,
                now,
                all_workspaces,
            },
        )
    })
}
