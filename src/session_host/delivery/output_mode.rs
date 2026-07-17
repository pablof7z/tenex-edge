//! Transport-owned output presentation, derived separately from message delivery.

#[cfg(test)]
use crate::session_host::transport::TransportKind;

/// Whether ordinary session output has no current presentation surface. An ACP
/// session is steerable yet headless; a direct non-PTY harness is headed even
/// though the daemon cannot steer it while idle.
pub(crate) fn session_is_headless(session: &crate::state::Session) -> bool {
    session.presentation_state == crate::state::PresentationState::Headless
}

#[cfg(test)]
pub(crate) fn headless_for_endpoint(
    kind: TransportKind,
    has_endpoint: bool,
    presentation: Option<crate::pty::PresentationSnapshot>,
) -> bool {
    mode_is_headless(kind, has_endpoint, presentation)
}

#[cfg(test)]
fn mode_is_headless(
    kind: TransportKind,
    has_endpoint: bool,
    presentation: Option<crate::pty::PresentationSnapshot>,
) -> bool {
    match kind {
        TransportKind::Acp => true,
        TransportKind::Pty => has_endpoint && presentation.is_some_and(|state| state.is_headless()),
    }
}
