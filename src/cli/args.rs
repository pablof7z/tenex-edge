use clap::{Parser, Subcommand};

use super::admin::{AgentAction, AgentsAction, ChannelsAction, InviteArgs, ProjectAction};
use super::debug::DebugAction;
use super::harness::HarnessAction;
use super::install::InstallArgs;
use super::messaging::{ChatAction, PublishArgs};
use super::tmux_cli::LaunchArgs;
use super::who::WhoArgs;

#[derive(Parser)]
#[command(
    name = "tenex-edge",
    about = "Citizenship for your agents: identity + awareness on the Nostr fabric."
)]
pub struct Cli {
    #[command(subcommand)]
    pub(super) cmd: Cmd,
}

#[derive(Subcommand)]
pub(super) enum Cmd {
    // session-start / session-end / turn-start / turn-check / turn-end are NOT
    // subcommands. They are hook-driven lifecycle steps invoked only through
    // `harness <name> hook --type <…>`, which parses the harness's stdin payload and calls the
    // corresponding private fn (session_start_inner / session_end / turn_start /
    // turn_check / turn_end). There is no host-facing way — or need — to invoke
    // them by hand.
    /// List agents currently visible in the project/channel.
    Who(WhoArgs),
    /// Write or read NIP-29 project chat.
    Chat {
        #[command(subcommand)]
        action: ChatAction,
    },
    /// Manage NIP-29 project groups (list, set description).
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Diagnose daemon relay and storage-path configuration.
    Doctor,
    /// Manage NIP-29 subgroup task channels under a project (create, join, leave, list, switch).
    Channels {
        #[command(subcommand)]
        action: ChannelsAction,
    },
    /// Manage the local agent keystore: agents that have a private key on THIS
    /// machine under `<edge_home>/agents/<slug>.json`. These are the identities
    /// you can spawn locally; project membership is governed separately by the
    /// codec (e.g. the NIP-29 group's member list), not here.
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },
    /// List invitable agents and prior session ids.
    Agents {
        #[command(subcommand)]
        action: Option<AgentsAction>,
    },
    /// Invite an agent or prior session into a channel. Use --agent for a fresh
    /// session, or --session to restore a prior context.
    Invite(InviteArgs),
    /// Hook integration and statusline for any supported agent harness.
    Harness {
        #[command(subcommand)]
        action: HarnessAction,
    },
    /// Publish a long-form proposal (kind:30023) from this agent's session.
    Publish(PublishArgs),
    /// Launch an agent harness in a new tmux session, with tmux chrome hidden.
    Launch(LaunchArgs),
    /// Stop the daemon and prevent hooks from restarting it.
    #[command(hide = true)]
    Stop,
    /// Local debugging tools for hook injection and command telemetry.
    #[command(hide = true)]
    Debug {
        #[command(subcommand)]
        action: DebugAction,
    },
    /// Detect local agent harnesses and wire tenex-edge's hook entries into each.
    #[command(hide = true)]
    Install(InstallArgs),
    /// Start the per-machine daemon in the foreground.
    #[command(name = "daemon", alias = "__daemon", hide = true)]
    Daemon,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{error::ErrorKind, Parser};

    fn parse_err(args: &[&str]) -> clap::Error {
        match Cli::try_parse_from(args) {
            Ok(_) => panic!("expected parse failure for {args:?}"),
            Err(err) => err,
        }
    }

    #[test]
    fn removed_tail_command_stays_unavailable() {
        let err = parse_err(&["tenex-edge", "tail", "--live"]);

        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
    }
}
