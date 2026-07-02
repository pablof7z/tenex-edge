use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub(in crate::cli) struct WhoArgs {
    #[arg(long)]
    project: Option<String>,
    /// Show agents across all projects (overrides --project / cwd resolution).
    #[arg(long)]
    all_projects: bool,
    /// Keep a full-screen live view open, refreshing automatically.
    #[arg(long)]
    live: bool,
}

pub(in crate::cli) fn who(args: WhoArgs) -> Result<()> {
    if args.live {
        super::who_live(args.project, args.all_projects)
    } else {
        super::who_once(args.project, args.all_projects)
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[test]
    fn who_all_projects_live_parse_with_owner_args() {
        let cli = crate::cli::args::Cli::try_parse_from([
            "tenex-edge",
            "who",
            "--all-projects",
            "--live",
        ])
        .expect("who parses");

        match cli.cmd {
            crate::cli::args::Cmd::Who(args) => {
                assert!(args.all_projects);
                assert!(args.live);
                assert_eq!(args.project, None);
            }
            _ => panic!("expected who command"),
        }
    }
}
