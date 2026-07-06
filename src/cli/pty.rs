use anyhow::{Context, Result};
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub(in crate::cli) enum PtyAction {
    /// List experimental portable-pty sessions.
    List,
    /// Attach to a session id or socket path.
    Attach { id: String },
    /// Inject text into a session id or socket path.
    Inject(InjectArgs),
    /// Resize a session PTY.
    Resize(ResizeArgs),
    /// Kill a session's child process.
    Kill { id: String },
}

#[derive(Args)]
pub(in crate::cli) struct InjectArgs {
    id: String,
    #[arg(long)]
    bracketed: bool,
    #[arg(long)]
    no_submit: bool,
    text: Option<String>,
}

#[derive(Args)]
pub(in crate::cli) struct ResizeArgs {
    id: String,
    #[arg(long)]
    rows: u16,
    #[arg(long)]
    cols: u16,
}

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
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

pub(in crate::cli) fn pty(action: PtyAction) -> Result<()> {
    match action {
        PtyAction::List => crate::pty::list(),
        PtyAction::Attach { id } => crate::pty::attach(&id),
        PtyAction::Inject(args) => {
            let text = match args.text {
                Some(text) => text,
                None => {
                    let mut text = String::new();
                    std::io::Read::read_to_string(&mut std::io::stdin(), &mut text)
                        .context("reading inject text from stdin")?;
                    text
                }
            };
            crate::pty::inject(&args.id, &text, args.bracketed, !args.no_submit)
        }
        PtyAction::Resize(args) => crate::pty::resize(&args.id, args.rows, args.cols),
        PtyAction::Kill { id } => crate::pty::kill(&id),
    }
}

pub(in crate::cli) fn pty_supervisor(args: PtySupervisorArgs) -> Result<()> {
    crate::pty::run_supervisor(crate::pty::SupervisorArgs {
        id: args.id,
        socket: args.socket,
        cwd: args.cwd,
        agent: args.agent,
        channel: args.channel,
        command: args.command,
    })
}
