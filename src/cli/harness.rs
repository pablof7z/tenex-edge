use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub(in crate::cli) enum HarnessAction {
    /// Handle a hook event from a supported agent harness.
    /// Reads hook JSON from stdin; emits context to inject into the model (if any).
    /// Usage: `tenex-edge harness hook <name> --type <hook-type>`
    /// Always exits 0 — a hook failure (daemon down, config missing, RPC
    /// timeout, …) is fabric plumbing, never something to surface to the
    /// harness or inject into the agent's context.
    Hook {
        /// Harness name: claude-code, codex, opencode, grok, …
        /// Run with name "help" to list known harnesses.
        harness: String,
        /// Hook type the harness fires: session-start, user-prompt-submit,
        /// post-tool-use, stop, session-end.
        #[arg(long = "type")]
        hook_type: String,
    },
    /// Render the one-line fabric statusline for a host's status bar.
    /// Reads the harness's statusline JSON payload on stdin (for `session_id`),
    /// prints one line, and always exits 0 — fails open when the daemon is down
    /// (and never spawns one).
    Statusline {
        /// Session id; if omitted, taken from the stdin payload.
        #[arg(long)]
        session: Option<String>,
        /// Emit tmux #[style] format strings instead of ANSI codes. Required
        /// when the output is consumed by tmux's status-format (#(...)).
        #[arg(long)]
        tmux: bool,
    },
}

impl HarnessAction {
    pub(in crate::cli) fn is_hook(&self) -> bool {
        matches!(self, Self::Hook { .. })
    }
}

pub(in crate::cli) async fn harness(action: HarnessAction) -> Result<()> {
    match action {
        HarnessAction::Hook { harness, hook_type } => {
            // Hooks fire on every turn of an unrelated harness session. An error
            // here (daemon down, config missing, RPC failure, …) must NEVER
            // surface as a nonzero exit or an injected error blob — that would
            // pollute the agent's context with fabric plumbing it didn't ask
            // about. Log it for our own debugging and always exit clean.
            if let Err(e) = super::hooks::hook_run(harness, hook_type).await {
                eprintln!("[tenex-edge] hook error (ignored): {e:#}");
            }
            Ok(())
        }
        HarnessAction::Statusline { session, tmux } => super::statusline::statusline(session, tmux),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn harness_statusline_args_parse_with_owner_args() {
        let cli = crate::cli::args::Cli::try_parse_from([
            "tenex-edge",
            "harness",
            "statusline",
            "--session",
            "s1",
            "--tmux",
        ])
        .expect("harness statusline parses");

        match cli.cmd {
            crate::cli::args::Cmd::Harness {
                action: HarnessAction::Statusline { session, tmux },
            } => {
                assert_eq!(session.as_deref(), Some("s1"));
                assert!(tmux);
            }
            _ => panic!("expected harness statusline command"),
        }
    }
}
