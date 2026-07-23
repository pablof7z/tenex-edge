use super::model::*;
use std::collections::{BTreeMap, BTreeSet};

mod channels;
use channels::{channels_by_parent, workspace_view};

pub(crate) struct AgentWhoInput<'a> {
    pub(crate) roots: &'a [String],
    pub(crate) self_name: &'a str,
    pub(crate) self_pubkey: &'a str,
    pub(crate) local_host: &'a str,
    pub(crate) backend_pubkey: &'a str,
    pub(crate) now: u64,
    pub(crate) headless: bool,
    pub(crate) active_channels: &'a BTreeSet<String>,
    pub(crate) expanded_workspaces: &'a BTreeSet<String>,
}

pub(super) fn build_agent_who(
    aggregation: &crate::who_aggregation::WhoAggregation,
    input: AgentWhoInput<'_>,
) -> anyhow::Result<AgentWhoView> {
    let by_parent = channels_by_parent(aggregation.channels.clone());
    let workspaces = input
        .roots
        .iter()
        .map(|root| workspace_view(aggregation, &by_parent, root, &input))
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(AgentWhoView {
        self_name: input.self_name.to_string(),
        self_host: input.local_host.to_string(),
        headless: input.headless,
        hosts: available_hosts(aggregation, input.roots),
        workspaces,
    })
}

fn available_hosts(
    aggregation: &crate::who_aggregation::WhoAggregation,
    roots: &[String],
) -> Vec<HostView> {
    let mut grouped = BTreeMap::<String, BTreeMap<String, String>>::new();
    let mut seen_pubkeys = BTreeSet::new();
    for profile in roots
        .iter()
        .flat_map(|root| aggregation.backend_profiles_for_root(Some(root)))
    {
        if !seen_pubkeys.insert(profile.pubkey.as_str()) {
            continue;
        }
        let host = profile.host.trim();
        if host.is_empty() {
            continue;
        }
        let agents = grouped.entry(host.to_string()).or_default();
        for (slug, about) in &profile.agents {
            agents
                .entry(format!("{slug}@{host}"))
                .or_insert_with(|| crate::agent_about::for_injection(about));
        }
    }
    grouped
        .into_iter()
        .map(|(name, agents)| HostView {
            name,
            agents: agents
                .into_iter()
                .map(|(reference, about)| AgentCapabilityView { reference, about })
                .collect(),
        })
        .collect()
}
