use anyhow::Result;

const MAX_MENU_ROWS: usize = 16;
const MAX_NAME_CHARS: usize = 30;
const MAX_DETAIL_CHARS: usize = 76;

pub(super) struct FreshAgentSelection {
    pub(super) slug: String,
    pub(super) retired_advertisements: Vec<String>,
}

pub(super) fn select_available() -> Result<Option<String>> {
    let cwd = std::env::current_dir().unwrap_or_default();
    let inventory = local_inventory(&cwd)?;
    if inventory.agents.is_empty() {
        println!("No available agents or harnesses.");
        for failure in inventory.failures {
            eprintln!("- unavailable: {failure}");
        }
        return Ok(None);
    }
    let agents = ordered_agents(&inventory);
    let labels = menu_labels(&agents);
    if !interactive_terminal() {
        println!("Available launch targets:");
        for label in labels {
            println!("- {label}");
        }
        return Ok(None);
    }

    let theme = dialoguer::theme::ColorfulTheme::default();
    let selected = dialoguer::FuzzySelect::with_theme(&theme)
        .with_prompt("Launch agent · type to filter")
        .items(&labels)
        .default(0)
        .max_length(MAX_MENU_ROWS)
        .vim_mode(true)
        .interact_opt()?;
    Ok(selected.map(|index| agents[index].slug.clone()))
}

pub(super) fn resolve_fresh_agent(
    requested: &str,
    cwd: &std::path::Path,
) -> Result<FreshAgentSelection> {
    let inventory = local_inventory(cwd)?;
    if let Some(selected) = inventory.find(requested) {
        return persist_selection(selected, &inventory);
    }

    let choices = inventory.profile_choices(requested);
    if choices.is_empty() {
        anyhow::bail!("no available agent or harness named {requested:?}");
    }
    if !interactive_terminal() {
        anyhow::bail!(
            "agent {requested:?} is available from multiple harnesses; choose {}",
            choices
                .iter()
                .map(|choice| format!("`mosaico launch {}`", choice.slug))
                .collect::<Vec<_>>()
                .join(" or ")
        );
    }
    let labels = choices
        .iter()
        .map(|choice| choice.harness.agent_slug())
        .collect::<Vec<_>>();
    let index = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!("Select harness for {requested}"))
        .items(&labels)
        .default(0)
        .interact()?;
    persist_selection(choices[index], &inventory)
}

fn interactive_terminal() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal() && std::io::stderr().is_terminal()
}

fn ordered_agents(
    inventory: &crate::agent_inventory::AgentInventory,
) -> Vec<&crate::agent_inventory::AvailableAgent> {
    let mut agents = inventory.agents.iter().collect::<Vec<_>>();
    agents.sort_by(|a, b| {
        source_rank(a.source)
            .cmp(&source_rank(b.source))
            .then_with(|| a.slug.to_lowercase().cmp(&b.slug.to_lowercase()))
    });
    agents
}

fn source_rank(source: crate::agent_inventory::AgentSource) -> u8 {
    match source {
        crate::agent_inventory::AgentSource::Harness => 0,
        crate::agent_inventory::AgentSource::Configured => 1,
        crate::agent_inventory::AgentSource::NativeProfile => 2,
    }
}

fn menu_labels(agents: &[&crate::agent_inventory::AvailableAgent]) -> Vec<String> {
    let name_width = agents
        .iter()
        .map(|agent| agent.slug.chars().count())
        .max()
        .unwrap_or(0)
        .min(MAX_NAME_CHARS);
    agents
        .iter()
        .map(|agent| menu_label(agent, name_width))
        .collect()
}

fn menu_label(agent: &crate::agent_inventory::AvailableAgent, name_width: usize) -> String {
    let source = match agent.source {
        crate::agent_inventory::AgentSource::Configured => "configured".to_string(),
        crate::agent_inventory::AgentSource::NativeProfile => {
            format!("{} profile", harness_label(agent.harness))
        }
        crate::agent_inventory::AgentSource::Harness => {
            format!("{} harness", harness_label(agent.harness))
        }
    };
    let criteria = compact(&agent.use_criteria);
    let detail = if criteria.is_empty() {
        source
    } else {
        format!("{source} · {criteria}")
    };
    let name = truncate(&agent.slug, name_width);
    format!(
        "{:<width$}  {}",
        name,
        truncate(&detail, MAX_DETAIL_CHARS),
        width = name_width
    )
}

fn harness_label(harness: crate::session::Harness) -> &'static str {
    match harness {
        crate::session::Harness::ClaudeCode => "Claude",
        crate::session::Harness::Codex => "Codex",
        crate::session::Harness::Opencode => "OpenCode",
        crate::session::Harness::Grok => "Grok",
        crate::session::Harness::Unknown => "Unknown",
    }
}

fn compact(value: &str) -> String {
    value
        .replace("\\n", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let prefix = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_none() {
        prefix
    } else {
        format!(
            "{}…",
            prefix
                .chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
        )
    }
}

fn persist_selection(
    selected: &crate::agent_inventory::AvailableAgent,
    inventory: &crate::agent_inventory::AgentInventory,
) -> Result<FreshAgentSelection> {
    let retired_advertisements = if selected.persist_binding {
        let retired = inventory
            .profile_choices(&selected.agent_slug)
            .into_iter()
            .map(|choice| choice.slug.clone())
            .collect();
        crate::identity::add_local_agent(
            &crate::config::mosaico_home(),
            &selected.agent_slug,
            &selected.bundle,
            None,
            crate::util::now_secs(),
        )?;
        retired
    } else {
        Vec::new()
    };
    Ok(FreshAgentSelection {
        slug: selected.agent_slug.clone(),
        retired_advertisements,
    })
}

fn local_inventory(cwd: &std::path::Path) -> Result<crate::agent_inventory::AgentInventory> {
    let config = crate::config::Config::load()?;
    let harnesses = crate::harness::HarnessesConfig::load()?;
    let catalog = crate::agent_catalog::AgentCatalog::discover(
        &crate::agent_catalog::DiscoveryRoots::installed()?,
        &[cwd.to_path_buf()],
    )?;
    Ok(crate::agent_inventory::AgentInventory::build(
        &crate::config::mosaico_home(),
        &config.available_harnesses,
        &harnesses,
        &catalog,
        Some(cwd),
    ))
}

#[cfg(test)]
#[path = "selection/tests.rs"]
mod tests;
