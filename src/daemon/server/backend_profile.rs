use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::daemon::server) struct HostAgent {
    pub(in crate::daemon::server) slug: String,
    pub(in crate::daemon::server) about: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::daemon::server) struct BackendProfileSnapshot {
    pub(in crate::daemon::server) agents: Vec<HostAgent>,
    pub(in crate::daemon::server) workspaces: Vec<String>,
    pub(in crate::daemon::server) failures: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(in crate::daemon::server) struct BackendProfilePublishReport {
    pub(in crate::daemon::server) agents: usize,
    pub(in crate::daemon::server) workspaces: usize,
    pub(in crate::daemon::server) failed: Vec<String>,
}

pub(in crate::daemon::server) fn rpc_backend_profile_refresh(
    state: &Arc<DaemonState>,
) -> Result<serde_json::Value> {
    state.refresh_agent_catalog()?;
    state.schedule_backend_profile_refresh();
    Ok(serde_json::json!({"scheduled": true}))
}

pub(in crate::daemon::server) async fn publish_backend_profile(
    state: &Arc<DaemonState>,
) -> Result<BackendProfilePublishReport> {
    let backend_keys = state
        .provider
        .management_keys()
        .context("backend profile requires a management key")?;
    let snapshot = backend_profile_snapshot(state)?;
    for failure in &snapshot.failures {
        tracing::error!(error = %failure, "agent capability is not advertisable");
    }
    let agent_count = snapshot.agents.len();
    let workspace_count = snapshot.workspaces.len();
    let profile = crate::domain::Profile::backend_named(
        backend_keys.public_key().to_hex(),
        format!("{} (mosaico)", state.host),
        state.host.clone(),
        state.owners.clone(),
    )
    .with_agents(
        snapshot
            .agents
            .into_iter()
            .map(|agent| (agent.slug, agent.about))
            .collect(),
    )
    .with_workspaces(snapshot.workspaces);
    state
        .provider
        .enqueue(&crate::domain::DomainEvent::Profile(profile), &backend_keys)
        .await?;
    Ok(BackendProfilePublishReport {
        agents: agent_count,
        workspaces: workspace_count,
        failed: snapshot.failures,
    })
}

impl DaemonState {
    pub(crate) fn schedule_backend_profile_refresh(self: &Arc<Self>) {
        let state = self.clone();
        tokio::spawn(async move {
            if let Err(error) = publish_backend_profile(&state).await {
                tracing::warn!(
                    error = %format!("{error:#}"),
                    "backend profile publish failed"
                );
            }
        });
    }
}

pub(in crate::daemon::server) fn backend_profile_snapshot(
    state: &Arc<DaemonState>,
) -> Result<BackendProfileSnapshot> {
    let workspaces = state.with_store(super::who::root_channels)?;
    let bindings = state.with_store(|store| {
        crate::daemon::workspace_path::WorkspacePathResolver::new(store).bindings()
    })?;
    let catalog = state.agent_catalog();
    let installed_harnesses = state.installed_harnesses();
    let mut failures = Vec::new();
    let harnesses = match crate::harness::HarnessesConfig::load() {
        Ok(config) => config,
        Err(error) => {
            failures.push(format!("harnesses.json: {error:#}"));
            crate::harness::HarnessesConfig::default()
        }
    };
    let mut agents = BTreeMap::<String, String>::new();
    merge_inventory(
        &mut agents,
        crate::agent_inventory::AgentInventory::build(
            &crate::config::mosaico_home(),
            &installed_harnesses,
            &harnesses,
            &catalog,
            None,
        ),
        &mut failures,
    );
    for binding in bindings {
        merge_inventory(
            &mut agents,
            crate::agent_inventory::AgentInventory::build(
                &crate::config::mosaico_home(),
                &installed_harnesses,
                &harnesses,
                &catalog,
                Some(std::path::Path::new(&binding.abs_path)),
            ),
            &mut failures,
        );
    }
    failures.sort();
    failures.dedup();
    Ok(BackendProfileSnapshot {
        agents: agents
            .into_iter()
            .map(|(slug, about)| HostAgent { slug, about })
            .collect(),
        workspaces,
        failures,
    })
}

fn merge_inventory(
    merged: &mut BTreeMap<String, String>,
    inventory: crate::agent_inventory::AgentInventory,
    failures: &mut Vec<String>,
) {
    failures.extend(inventory.failures);
    for agent in inventory.agents {
        let about = agent.use_criteria;
        let entry = merged.entry(agent.slug).or_default();
        if entry.is_empty() {
            *entry = about;
        }
    }
}

#[cfg(test)]
#[path = "backend_profile/tests.rs"]
mod tests;
