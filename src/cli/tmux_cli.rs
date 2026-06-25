pub mod attach;
#[cfg(test)]
mod tests;
pub mod tui_model;
pub mod tui_render;
pub mod tui_run;
pub mod tui_terminal;
/// TMUX control-plane integration
pub mod verbs;

// Re-export public items for external callers (cli.rs)
pub(super) use tui_run::tmux_tui;
pub(super) use verbs::launch;

use anyhow::Result;

// ── tmux_run ──────────────────────────────────────────────────────────────────

/// Entry point for `tenex-edge tmux <action>`.
pub(super) async fn tmux_run(action: super::TmuxAction) -> Result<()> {
    use super::TmuxAction;
    use verbs::{tmux_attach, tmux_resume, tmux_send, tmux_spawn, tmux_status};

    match action {
        TmuxAction::Status => tmux_status().await,
        TmuxAction::Send { session } => tmux_send(session).await,
        TmuxAction::Spawn { agent, project } => tmux_spawn(agent, project).await,
        TmuxAction::Attach { session } => tmux_attach(session).await,
        TmuxAction::Resume { session } => tmux_resume(session).await,
        // The narrow long-running "sidebar" reuses the interactive session
        // switcher (lists project sessions, lets the user switch). It runs in
        // popup-style switch-and-exit mode so selecting a session hands the
        // tmux client over to it.
        TmuxAction::Sidebar { .. } => tmux_tui(true),
    }
}
