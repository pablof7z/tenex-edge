use anyhow::{bail, Context as _, Result};
use clap::Parser;
use std::io::{self, Read as _};

pub(in crate::cli) struct LaunchRequest {
    pub(in crate::cli) agent: String,
    pub(in crate::cli) channel: Option<String>,
    pub(in crate::cli) session_name: Option<String>,
    pub(in crate::cli) prompt: Option<String>,
    pub(in crate::cli) extra_args: Vec<String>,
}

impl LaunchRequest {
    pub(in crate::cli) fn from_external(args: Vec<String>) -> Result<Self> {
        let parsed =
            FallbackLaunchArgs::try_parse_from(std::iter::once("mosaico".to_string()).chain(args))
                .map_err(anyhow::Error::new)?;
        Ok(Self {
            agent: parsed.target,
            channel: parsed.channel,
            session_name: parsed.session_name,
            prompt: resolve_initial_prompt(parsed.prompt)?,
            extra_args: parsed.extra_args,
        })
    }
}

#[derive(Parser)]
#[command(name = "mosaico")]
struct FallbackLaunchArgs {
    /// Existing session or available agent name.
    target: String,
    /// Opening user prompt for a fresh session. Use "-" to read from stdin.
    #[arg(value_name = "PROMPT")]
    prompt: Option<String>,
    /// Channel name, or omit its value to open the channel picker.
    #[arg(long, num_args(0..=1), default_missing_value = "")]
    channel: Option<String>,
    /// Public name for this session.
    #[arg(long = "name", value_name = "NAME")]
    session_name: Option<String>,
    /// Arguments appended to the selected harness command.
    #[arg(last = true, value_name = "ARG")]
    extra_args: Vec<String>,
}

fn resolve_initial_prompt(raw: Option<String>) -> Result<Option<String>> {
    match raw {
        Some(prompt) if prompt == "-" => read_stdin_prompt().map(Some),
        Some(prompt) if prompt.is_empty() => bail!("prompt must not be empty"),
        Some(prompt) => Ok(Some(prompt)),
        None => Ok(None),
    }
}

fn read_stdin_prompt() -> Result<String> {
    let mut prompt = String::new();
    io::stdin()
        .read_to_string(&mut prompt)
        .context("failed to read prompt from stdin")?;
    let prompt = strip_single_trailing_newline(prompt);
    if prompt.is_empty() {
        bail!("prompt from stdin was empty");
    }
    Ok(prompt)
}

fn strip_single_trailing_newline(mut value: String) -> String {
    if value.ends_with('\n') {
        value.pop();
        if value.ends_with('\r') {
            value.pop();
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_parses_launch_options_and_separator_args() {
        let request = LaunchRequest::from_external(vec![
            "codex".into(),
            "hello".into(),
            "--channel".into(),
            "ops".into(),
            "--name".into(),
            "builder".into(),
            "--".into(),
            "--yolo".into(),
        ])
        .unwrap();

        assert_eq!(request.agent, "codex");
        assert_eq!(request.prompt.as_deref(), Some("hello"));
        assert_eq!(request.channel.as_deref(), Some("ops"));
        assert_eq!(request.session_name.as_deref(), Some("builder"));
        assert_eq!(request.extra_args, ["--yolo"]);
    }

    #[test]
    fn provider_args_require_the_separator() {
        assert!(LaunchRequest::from_external(vec!["codex".into(), "--yolo".into()]).is_err());
    }

    #[test]
    fn workspace_override_is_not_part_of_direct_launch() {
        assert!(LaunchRequest::from_external(vec![
            "codex".into(),
            "--workspace".into(),
            "other".into(),
        ])
        .is_err());
    }
}
