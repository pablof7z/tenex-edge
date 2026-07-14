use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub(in crate::cli) struct PtySupervisorArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    socket: std::path::PathBuf,
    #[arg(long)]
    cwd: std::path::PathBuf,
    #[arg(long)]
    agent: String,
    #[arg(long)]
    channel: Option<String>,
    #[arg(long)]
    session_name: Option<String>,
    #[arg(long)]
    ephemeral: bool,
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

pub(in crate::cli) fn pty_supervisor(args: PtySupervisorArgs) -> Result<()> {
    crate::pty::run_supervisor(crate::pty::SupervisorArgs {
        id: args.id,
        socket: args.socket,
        cwd: args.cwd,
        agent: args.agent,
        channel: args.channel,
        session_name: args.session_name,
        ephemeral: args.ephemeral,
        command: args.command,
    })
}
