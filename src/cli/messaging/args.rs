use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub(in crate::cli) struct PublishArgs {
    /// Proposal title.
    #[arg(long)]
    title: String,
    /// Proposal body (Markdown). Use "-" or omit to read from stdin.
    #[arg(long = "message", value_name = "BODY")]
    message: Option<String>,
    /// Stable addressable identifier (the kind:30023 `d` tag). Reuse the same
    /// value to publish a REVISION that supersedes a prior proposal at the same
    /// address. Omit to mint a fresh id (a new proposal).
    #[arg(long = "d", value_name = "IDENTIFIER")]
    d: Option<String>,
    /// My session id; if omitted, resolved from the current directory.
    #[arg(long)]
    session: Option<String>,
}

pub(in crate::cli) async fn publish(args: PublishArgs) -> Result<()> {
    let body = super::resolve_send_message_body(args.message)?;
    super::publish_proposal(args.title, body, args.d, args.session).await
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[test]
    fn publish_args_parse_with_owner_args() {
        let cli = crate::cli::args::Cli::try_parse_from([
            "tenex-edge",
            "publish",
            "--title",
            "T",
            "--message",
            "body",
            "--d",
            "proposal-1",
            "--session",
            "session-1",
        ])
        .unwrap();

        match cli.cmd {
            crate::cli::args::Cmd::Publish(args) => {
                assert_eq!(args.title, "T");
                assert_eq!(args.message.as_deref(), Some("body"));
                assert_eq!(args.d.as_deref(), Some("proposal-1"));
                assert_eq!(args.session.as_deref(), Some("session-1"));
            }
            _ => panic!("expected publish command"),
        }
    }
}
