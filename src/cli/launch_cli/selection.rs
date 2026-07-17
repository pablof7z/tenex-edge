use anyhow::Result;

pub(super) struct FreshAgentSelection {
    pub(super) slug: String,
    pub(super) retired_advertisements: Vec<String>,
}

pub(super) fn list_available() -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_default();
    let inventory = local_inventory(&cwd)?;
    if inventory.agents.is_empty() {
        println!("No available agents or harnesses.");
        for failure in inventory.failures {
            eprintln!("- unavailable: {failure}");
        }
        return Ok(());
    }
    println!("Available agents:");
    for agent in inventory.agents {
        let criteria = agent.use_criteria.trim();
        if criteria.is_empty() {
            println!("- {}", agent.slug);
        } else {
            println!("- {}: {criteria}", agent.slug);
        }
    }
    Ok(())
}

pub(super) fn resolve_fresh_agent(
    requested: &str,
    cwd: &std::path::Path,
) -> Result<FreshAgentSelection> {
    let home = crate::config::mosaico_home();
    if crate::identity::is_configured(&home, requested) {
        return Ok(FreshAgentSelection {
            slug: requested.to_string(),
            retired_advertisements: Vec::new(),
        });
    }
    let inventory = local_inventory(cwd)?;
    if let Some(selected) = inventory.find(requested) {
        return persist_selection(selected, &inventory);
    }

    let choices = inventory.profile_choices(requested);
    if choices.is_empty() {
        anyhow::bail!("no available agent or harness named {requested:?}");
    }
    use std::io::IsTerminal;
    if !(std::io::stdin().is_terminal() && std::io::stdout().is_terminal()) {
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
mod tests {
    use super::*;
    use crate::test_env::EnvGuard;

    fn write(path: &std::path::Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn explicit_conflict_combination_persists_canonical_profile_binding() {
        let root = tempfile::tempdir().unwrap();
        let mosaico_home = root.path().join("mosaico");
        let codex_home = root.path().join(".codex");
        let mut env = EnvGuard::set("HOME", root.path());
        env.set_var("MOSAICO_HOME", &mosaico_home);
        env.set_var("MOSAICO_ISOLATED_HOME_OK", "1");
        env.set_var("CODEX_HOME", &codex_home);
        env.set_var("XDG_CONFIG_HOME", root.path().join(".config"));
        write(
            &mosaico_home.join("config.json"),
            r#"{"availableHarnesses":["claude","codex"]}"#,
        );
        write(
            &mosaico_home.join("harnesses.json"),
            r#"{
              "claude-pty":{"harness":"claude","transport":"pty"},
              "codex-pty":{"harness":"codex","transport":"pty"}
            }"#,
        );
        write(
            &codex_home.join("agents/writer.toml"),
            "name='writer'\ndescription='Writes'\ndeveloper_instructions='Write'",
        );
        write(
            &root.path().join(".claude/agents/writer.md"),
            "---\nname: writer\ndescription: Writes\n---\nWrite",
        );

        let selection = resolve_fresh_agent("writer-codex", root.path()).unwrap();

        assert_eq!(selection.slug, "writer");
        assert_eq!(
            selection.retired_advertisements,
            ["writer-claude", "writer-codex"]
        );
        let binding: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(mosaico_home.join("agents/writer.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(binding["slug"], "writer");
        assert_eq!(binding["harness"], "codex-pty");
        assert!(binding.get("profile").is_none());
    }
}
