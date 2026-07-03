use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub(super) action: Option<ConfigAction>,
}

#[derive(Subcommand)]
pub(super) enum ConfigAction {
    /// Add, edit, or remove provider credentials (OpenRouter API key, Ollama
    /// base URL, ...).
    Providers,
    /// Assign a model to a role, fuzzy-searching the live model list from a
    /// configured provider.
    Models,
}
