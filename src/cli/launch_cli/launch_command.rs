use std::io::IsTerminal;

use anyhow::{bail, Context, Result};

use crate::identity::{adapt_argv_for_slug, LaunchCommand, SpawnAgentEntry};

#[derive(Clone)]
struct CommandSuggestion {
    label: String,
    command: LaunchCommand,
}

pub(super) fn resolve_launch_command(
    agent: &str,
    command_name: Option<&str>,
    launch_args: &[String],
) -> Result<Vec<String>> {
    let edge_home = crate::config::edge_home();
    let agents = crate::identity::list_local_agents(&edge_home);
    let commands = agents
        .iter()
        .find(|(slug, _, _, _)| slug == agent)
        .map(|(_, commands, _, _)| commands.clone())
        .unwrap_or_default();

    if !commands.is_empty() {
        return choose_configured_command(agent, &commands, command_name);
    }
    if let Some(name) = command_name {
        bail!("agent {agent:?} has no configured commands; cannot select {name:?}");
    }
    ensure_tty(agent)?;

    let suggestions = missing_command_suggestions(agent, &agents);
    let command = pick_missing_command(agent, suggestions, launch_args)?;
    crate::identity::add_local_agent_with_commands(
        &edge_home,
        agent,
        vec![command.clone()],
        crate::util::now_secs(),
    )?;
    Ok(command.argv)
}

fn choose_configured_command(
    agent: &str,
    commands: &[LaunchCommand],
    command_name: Option<&str>,
) -> Result<Vec<String>> {
    if let Some(name) = command_name {
        return commands
            .iter()
            .find(|command| command.name == name)
            .map(|command| command.argv.clone())
            .with_context(|| {
                let names = commands
                    .iter()
                    .map(|command| command.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("agent {agent:?} has no command named {name:?}; available: {names}")
            });
    }
    if commands.len() == 1 {
        return Ok(commands[0].argv.clone());
    }
    ensure_tty(agent)?;
    let labels = commands
        .iter()
        .map(LaunchCommand::display)
        .collect::<Vec<_>>();
    let idx = dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select launch command")
        .items(&labels)
        .default(0)
        .interact()?;
    Ok(commands[idx].argv.clone())
}

fn ensure_tty(agent: &str) -> Result<()> {
    if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        return Ok(());
    }
    bail!(
        "launch command selection for agent {agent:?} needs a TTY; pass \
         --command-name <name> for a configured command or -c <command> as an override"
    )
}

fn pick_missing_command(
    agent: &str,
    suggestions: Vec<CommandSuggestion>,
    launch_args: &[String],
) -> Result<LaunchCommand> {
    let custom_label = "Custom command...".to_string();
    let mut labels = suggestions
        .iter()
        .map(|suggestion| suggestion.label.clone())
        .collect::<Vec<_>>();
    labels.push(custom_label);

    eprintln!("Agent {agent:?} has no configured commands.");
    let idx = dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select a launch command to save")
        .items(&labels)
        .default(0)
        .interact()?;

    if idx < suggestions.len() {
        return Ok(suggestions[idx].command.clone());
    }
    prompt_custom_command(launch_args)
}

fn prompt_custom_command(launch_args: &[String]) -> Result<LaunchCommand> {
    let name: String = dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Command name")
        .default(crate::identity::DEFAULT_COMMAND_NAME.to_string())
        .interact_text()?;
    let prompt = if launch_args.is_empty() {
        "Command".to_string()
    } else {
        format!(
            "Command to save ({} appended for this launch)",
            display_argv(launch_args)
        )
    };
    let raw: String = dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()?;
    let argv = shlex::split(&raw).unwrap_or_else(|| vec![raw]);
    LaunchCommand::new(name, argv).context("command name and argv must be non-empty")
}

pub(super) fn extra_args_without_duplicate_suffix(
    base_command: &[String],
    extra_args: Vec<String>,
) -> Vec<String> {
    if !extra_args.is_empty() && base_command.ends_with(extra_args.as_slice()) {
        Vec::new()
    } else {
        extra_args
    }
}

pub(super) fn append_launch_args(
    mut base_command: Vec<String>,
    extra_args: &[String],
) -> Vec<String> {
    base_command.extend(extra_args.iter().cloned());
    base_command
}

pub(super) fn display_argv(argv: &[String]) -> String {
    argv.iter()
        .map(|arg| {
            if arg
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':'))
            {
                arg.clone()
            } else {
                format!("'{}'", arg.replace('\'', "'\\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn missing_command_suggestions(
    target_slug: &str,
    agents: &[SpawnAgentEntry],
) -> Vec<CommandSuggestion> {
    let local = agent_command_suggestions(target_slug, agents);
    if !local.is_empty() {
        return local;
    }
    builtin_command_suggestions(target_slug)
}

fn agent_command_suggestions(
    target_slug: &str,
    agents: &[SpawnAgentEntry],
) -> Vec<CommandSuggestion> {
    let mut out = Vec::new();
    for (source_slug, commands, _, _) in agents {
        if source_slug == target_slug {
            continue;
        }
        for command in commands {
            let argv = adapt_argv_for_slug(&command.argv, source_slug, target_slug);
            let Some(command) = LaunchCommand::new(command.name.clone(), argv) else {
                continue;
            };
            out.push(CommandSuggestion {
                label: format!("@{source_slug} {}", command.display()),
                command,
            });
        }
    }
    out
}

fn builtin_command_suggestions(target_slug: &str) -> Vec<CommandSuggestion> {
    let mut commands = crate::session_host::builtin_spawn_commands();
    commands.sort_by_key(|command| {
        if command.name == target_slug {
            (0, command.name.clone())
        } else {
            (1, command.name.clone())
        }
    });
    commands
        .into_iter()
        .map(|command| CommandSuggestion {
            label: format!("built-in {}", command.display()),
            command,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    fn command(name: &str, parts: &[&str]) -> LaunchCommand {
        LaunchCommand::new(name, argv(parts)).unwrap()
    }

    #[test]
    fn suggestions_adapt_other_agent_commands() {
        let agents = vec![(
            "poppy".to_string(),
            vec![command(
                "file",
                &["runner", "--file", "/home/me/poppy.json"],
            )],
            None,
            None,
        )];

        let suggestions = agent_command_suggestions("newagent", &agents);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].command.name, "file");
        assert_eq!(
            suggestions[0].command.argv,
            argv(&["runner", "--file", "/home/me/newagent.json"])
        );
    }

    #[test]
    fn suggestions_ignore_old_singular_command_shape() {
        let agents = vec![("legacy".to_string(), vec![], None, None)];
        assert!(agent_command_suggestions("newagent", &agents).is_empty());
    }

    #[test]
    fn builtins_prefer_matching_target_slug() {
        let suggestions = builtin_command_suggestions("codex");
        assert_eq!(suggestions[0].command.name, "codex");
        assert_eq!(suggestions[0].command.argv, argv(&["codex"]));
    }

    #[test]
    fn duplicate_extra_args_are_not_appended_twice() {
        let base = argv(&["codex", "--yolo"]);
        let extra = argv(&["--yolo"]);
        assert!(extra_args_without_duplicate_suffix(&base, extra).is_empty());
    }

    #[test]
    fn distinct_extra_args_are_preserved() {
        let base = argv(&["codex", "--model", "gpt-5"]);
        let extra = argv(&["--yolo"]);
        assert_eq!(
            extra_args_without_duplicate_suffix(&base, extra),
            argv(&["--yolo"])
        );
    }

    #[test]
    fn display_argv_quotes_shell_sensitive_args() {
        assert_eq!(
            display_argv(&argv(&["codex", "--profile", "work profile"])),
            "codex --profile 'work profile'"
        );
    }
}
