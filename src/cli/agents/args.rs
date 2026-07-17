use clap::{Args, Subcommand};

#[derive(Args)]
pub(in crate::cli) struct AgentsArgs {
    #[command(subcommand)]
    pub(super) action: Option<AgentAction>,
}

#[derive(Subcommand)]
pub(super) enum AgentAction {
    /// Print every configured agent and available native/default agent.
    List,
    /// Create or update a configured agent binding.
    Add {
        slug: String,
        #[arg(long, value_name = "BUNDLE")]
        harness: String,
        #[arg(long, value_name = "PROFILE")]
        profile: Option<String>,
    },
    /// Permanently delete a configured agent JSON file.
    Remove { slug: String },
}
