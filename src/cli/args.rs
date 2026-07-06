use clap::{Parser, Subcommand};

use super::admin::{AgentAction, AgentsAction, ChannelsAction, InviteArgs, ProjectAction};
use super::config::ConfigArgs;
use super::debug::DebugAction;
use super::explain::ExplainArgs;
use super::harness::HarnessAction;
use super::install::InstallArgs;
use super::launch_cli::LaunchArgs;
use super::messaging::{ChatAction, PublishArgs};
use super::probe::ProbeArgs;
use super::pty::{PtyAction, PtySupervisorArgs};
use super::validate::ValidateArgs;
use super::who::WhoArgs;

#[derive(Parser)]
#[command(
    name = "tenex-edge",
    about = "An identity and awareness fabric for the coding agents you already run."
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
    /// Explain a published artifact: the reconciler receipt + the exact LLM
    /// inputs (system prompt, transcript slice, model, raw response) behind it.
    Explain(ExplainArgs),
    /// Validate a surface, handle, event/message/recipient target, awareness
    /// target, channel/readiness/readiness_attempt target, commit target, fact,
    /// or replay capsule with explanations.
    Validate(ValidateArgs),
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
    /// Interactively configure model providers and role-to-model assignments.
    Config(ConfigArgs),
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
    /// Launch an agent harness in a reattachable portable-pty session.
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
    /// Diagnostic probe over the reconciler frontier: stats/oracle/simulate/why/state.
    #[command(hide = true)]
    Probe(ProbeArgs),
    /// Experimental portable-pty supervisor test surface.
    #[command(hide = true)]
    Pty {
        #[command(subcommand)]
        action: PtyAction,
    },
    /// Internal portable-pty supervisor process.
    #[command(name = "__pty-supervisor", hide = true)]
    PtySupervisor(PtySupervisorArgs),
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
