use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub(in crate::cli) struct LaunchArgs {
    /// Agent slug: "claude", "codex", "opencode", or a local custom agent.
    slug: String,
    /// Project slug; defaults to project resolved from current directory.
    #[arg(long)]
    project: Option<String>,
    /// Channel name to scope this agent into; resolved to its opaque id and
    /// created if absent. Omit the value (`--channel` with no argument) to
    /// open an interactive fuzzy picker over all known rooms for the project.
    /// When per-session rooms are disabled (the default), omitting `--channel`
    /// entirely also opens the picker; with per-session rooms enabled, omitting
    /// it mints a fresh per-session room instead. The daemon's tenexPrivateKey
    /// adds the agent as a member; if the same derived pubkey is already in the
    /// group a fresh session produces a distinct key via a new anchor, acting
    /// as a second personality.
    #[arg(long, num_args(0..=1), default_missing_value = "")]
    channel: Option<String>,
    /// Override the entire launch command for this launch (shell-word split).
    /// Example: `-c 'ollama launch claude -- --dangerously-skip-permissions'`
    #[arg(short = 'c', long = "command", value_name = "COMMAND")]
    command_str: Option<String>,
    /// Select a named command from the agent file's `commands` list.
    #[arg(long = "command-name", value_name = "NAME")]
    command_name: Option<String>,
    /// Extra args passed after `--`; appended to the launch command.
    /// Example: `tenex-edge launch codex -- --yolo`
    #[arg(last = true, value_name = "ARGS")]
    extra_args: Vec<String>,
}

pub(in crate::cli) async fn launch(args: LaunchArgs) -> Result<()> {
    let override_command = args
        .command_str
        .map(|s| shlex::split(&s).unwrap_or_else(|| vec![s]))
        .unwrap_or_default();
    super::verbs::launch(
        args.slug,
        args.project,
        args.channel,
        args.command_name,
        override_command,
        args.extra_args,
    )
    .await
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[test]
    fn launch_channel_tristate_is_explicit_contract() {
        let omitted = crate::cli::args::Cli::try_parse_from(["tenex-edge", "launch", "codex"])
            .expect("launch without channel parses");
        let picker =
            crate::cli::args::Cli::try_parse_from(["tenex-edge", "launch", "codex", "--channel"])
                .expect("launch with channel picker parses");
        let named = crate::cli::args::Cli::try_parse_from([
            "tenex-edge",
            "launch",
            "codex",
            "--channel",
            "ops",
        ])
        .expect("launch with named channel parses");

        let channel = |cli: crate::cli::args::Cli| match cli.cmd {
            crate::cli::args::Cmd::Launch(args) => args.channel,
            _ => panic!("expected launch command"),
        };

        assert_eq!(channel(omitted), None);
        assert_eq!(channel(picker).as_deref(), Some(""));
        assert_eq!(channel(named).as_deref(), Some("ops"));
    }

    #[test]
    fn launch_command_name_parses_independently_from_override() {
        let cli = crate::cli::args::Cli::try_parse_from([
            "tenex-edge",
            "launch",
            "codex",
            "--command-name",
            "safe",
        ])
        .expect("launch with command name parses");

        match cli.cmd {
            crate::cli::args::Cmd::Launch(args) => {
                assert_eq!(args.command_name.as_deref(), Some("safe"));
                assert!(args.command_str.is_none());
            }
            _ => panic!("expected launch command"),
        }
    }
}
