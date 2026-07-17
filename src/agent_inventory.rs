//! One launch inventory shared by the CLI, daemon roster, and spawn resolver.

use crate::agent_catalog::{AgentCapability, AgentCatalog};
use crate::harness::HarnessesConfig;
use crate::session::Harness;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentSource {
    Configured,
    NativeProfile,
    Harness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AvailableAgent {
    /// Public selection/advertisement name. Conflicts carry a harness suffix.
    pub(crate) slug: String,
    /// Canonical agent/profile name stored in sessions and agent JSON.
    pub(crate) agent_slug: String,
    pub(crate) bundle: String,
    pub(crate) harness: Harness,
    pub(crate) use_criteria: String,
    pub(crate) available_since: u64,
    pub(crate) source: AgentSource,
    /// Selecting a suffixed conflict option makes the choice durable.
    pub(crate) persist_binding: bool,
}

#[derive(Debug, Default)]
pub(crate) struct AgentInventory {
    pub(crate) agents: Vec<AvailableAgent>,
    pub(crate) failures: Vec<String>,
}

impl AgentInventory {
    pub(crate) fn build(
        mosaico_home: &Path,
        available_harnesses: &[Harness],
        harnesses: &HarnessesConfig,
        catalog: &AgentCatalog,
        workspace: Option<&Path>,
    ) -> Self {
        let mut inventory = Self::default();
        let mut configured = BTreeSet::new();
        let created = crate::identity::list_invitable_agents(mosaico_home)
            .into_iter()
            .map(|(slug, _, created_at)| (slug, created_at))
            .collect::<BTreeMap<_, _>>();

        for (slug, bundle, _, byline) in crate::identity::list_local_agents(mosaico_home) {
            match crate::harness::bundle_harness_with(harnesses, &bundle) {
                Ok(harness) => {
                    configured.insert(slug.clone());
                    inventory.agents.push(AvailableAgent {
                        slug: slug.clone(),
                        agent_slug: slug.clone(),
                        bundle,
                        harness,
                        use_criteria: byline.unwrap_or_default(),
                        available_since: created.get(&slug).copied().unwrap_or(0),
                        source: AgentSource::Configured,
                        persist_binding: false,
                    });
                }
                Err(error) => inventory.failures.push(format!("{slug}: {error:#}")),
            }
        }

        for capability in catalog.capabilities(workspace) {
            if !configured.contains(&capability.slug) {
                inventory.add_capability(capability, available_harnesses, harnesses);
            }
        }
        inventory.add_harnesses(available_harnesses, harnesses);
        inventory.agents.sort_by(|a, b| a.slug.cmp(&b.slug));
        inventory.failures.sort();
        inventory.failures.dedup();
        inventory
    }

    pub(crate) fn find(&self, slug: &str) -> Option<&AvailableAgent> {
        self.agents.iter().find(|agent| agent.slug == slug)
    }

    pub(crate) fn profile_choices(&self, slug: &str) -> Vec<&AvailableAgent> {
        self.agents
            .iter()
            .filter(|agent| {
                agent.source == AgentSource::NativeProfile
                    && agent.persist_binding
                    && agent.agent_slug == slug
            })
            .collect()
    }

    fn add_capability(
        &mut self,
        capability: AgentCapability,
        available_harnesses: &[Harness],
        harnesses: &HarnessesConfig,
    ) {
        let mut choices = Vec::new();
        for profile in capability.profiles {
            if !available_harnesses.contains(&profile.harness) {
                continue;
            }
            match crate::harness::native_bundle_with(harnesses, profile.harness) {
                Ok(bundle) => choices.push((profile.harness, bundle)),
                Err(error) => self
                    .failures
                    .push(format!("{}: {error:#}", capability.slug)),
            }
        }
        let conflicted = choices.len() > 1;
        for (harness, bundle) in choices {
            let slug = if conflicted {
                format!("{}-{}", capability.slug, harness.agent_slug())
            } else {
                capability.slug.clone()
            };
            self.agents.push(AvailableAgent {
                slug,
                agent_slug: capability.slug.clone(),
                bundle,
                harness,
                use_criteria: capability.use_criteria.clone(),
                available_since: capability.available_since,
                source: AgentSource::NativeProfile,
                persist_binding: conflicted,
            });
        }
    }

    fn add_harnesses(&mut self, available: &[Harness], harnesses: &HarnessesConfig) {
        for harness in available {
            let slug = harness.agent_slug();
            if self.agents.iter().any(|agent| agent.slug == slug) {
                continue;
            }
            match crate::harness::native_bundle_with(harnesses, *harness) {
                Ok(bundle) => self.agents.push(AvailableAgent {
                    slug: slug.to_string(),
                    agent_slug: slug.to_string(),
                    bundle,
                    harness: *harness,
                    use_criteria: String::new(),
                    available_since: 0,
                    source: AgentSource::Harness,
                    persist_binding: false,
                }),
                Err(error) => self.failures.push(format!("{slug}: {error:#}")),
            }
        }
    }
}

#[cfg(test)]
#[path = "agent_inventory/tests.rs"]
mod tests;
