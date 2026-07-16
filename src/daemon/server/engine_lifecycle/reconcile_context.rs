use crate::daemon::server::DaemonState;
use crate::state::Session;

pub(super) fn parent_hint(state: &DaemonState, session: &Session) -> Option<String> {
    let relay_parent =
        state.with_store(|store| store.channel_parent(&session.channel_h).ok().flatten());
    crate::fabric::nip29::readiness::effective_parent_hint(
        relay_parent,
        Some(&session.readiness_parent),
        &session.channel_h,
    )
}

pub(super) fn workspace(state: &DaemonState, session: &Session) -> String {
    state
        .with_store(|store| store.root_channel_of(&session.channel_h).ok().flatten())
        .or_else(|| (!session.work_root.is_empty()).then(|| session.work_root.clone()))
        .unwrap_or_else(|| session.channel_h.clone())
}
