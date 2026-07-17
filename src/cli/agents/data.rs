use crate::agent_catalog::{AgentCatalog, NativeAgentProfile};
use crate::harness::{HarnessesConfig, Transport};
use crate::session::Harness;
use anyhow::Result;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AgentKind {
    Configured,
    NativeProfile,
    Generic,
}

#[derive(Clone, Debug)]
pub(super) struct AgentRow {
    pub(super) slug: String,
    pub(super) description: String,
    pub(super) harness: Harness,
    pub(super) bundle: Option<String>,
    pub(super) transport: Option<Transport>,
    pub(super) profile: Option<String>,
    pub(super) per_session_key: Option<bool>,
    pub(super) kind: AgentKind,
    pub(super) native_profile: Option<NativeAgentProfile>,
}

pub(super) fn load() -> Result<Vec<AgentRow>> {
    let cwd = std::env::current_dir()?;
    let home = crate::config::mosaico_home();
    let config = crate::config::Config::load()?;
    let harnesses = HarnessesConfig::load()?;
    let catalog = AgentCatalog::discover(
        &crate::agent_catalog::DiscoveryRoots::installed()?,
        std::slice::from_ref(&cwd),
    )?;
    Ok(build(
        &home,
        &config.available_harnesses,
        &harnesses,
        &catalog,
        &cwd,
    ))
}

fn build(
    home: &Path,
    available_harnesses: &[Harness],
    harnesses: &HarnessesConfig,
    catalog: &AgentCatalog,
    workspace: &Path,
) -> Vec<AgentRow> {
    let bylines = crate::identity::list_local_agents(home)
        .into_iter()
        .map(|(slug, _, _, byline)| (slug, byline))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut profiles = catalog
        .capabilities(Some(workspace))
        .into_iter()
        .flat_map(|capability| capability.profiles)
        .filter(|profile| available_harnesses.contains(&profile.harness))
        .collect::<Vec<_>>();
    let mut rows = Vec::new();

    for agent in crate::identity::list_local_agent_details(home) {
        let bundle = harnesses.get(&agent.harness);
        let harness = bundle
            .map(|bundle| bundle.harness)
            .unwrap_or(Harness::Unknown);
        let matching_profile = profiles
            .iter()
            .position(|profile| profile.slug == agent.slug && profile.harness == harness)
            .map(|index| profiles.remove(index));
        let description = bylines
            .get(&agent.slug)
            .and_then(Clone::clone)
            .or_else(|| matching_profile.as_ref().map(|p| p.use_criteria.clone()))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "Configured agent".to_string());
        rows.push(AgentRow {
            slug: agent.slug,
            description,
            harness,
            bundle: Some(agent.harness),
            transport: bundle.map(|bundle| bundle.transport),
            profile: agent.profile,
            per_session_key: Some(agent.per_session_key),
            kind: AgentKind::Configured,
            native_profile: matching_profile,
        });
    }

    rows.extend(profiles.into_iter().map(|profile| {
        let bundle = preferred_bundle(harnesses, profile.harness, true);
        let transport = bundle
            .as_deref()
            .and_then(|name| harnesses.get(name))
            .map(|bundle| bundle.transport);
        AgentRow {
            slug: profile.slug.clone(),
            description: profile.use_criteria.clone(),
            harness: profile.harness,
            bundle,
            transport,
            profile: None,
            per_session_key: None,
            kind: AgentKind::NativeProfile,
            native_profile: Some(profile),
        }
    }));

    for harness in available_harnesses {
        let slug = harness.agent_slug();
        if rows.iter().any(|row| row.slug == slug) {
            continue;
        }
        let bundle = preferred_bundle(harnesses, *harness, false);
        let transport = bundle
            .as_deref()
            .and_then(|name| harnesses.get(name))
            .map(|bundle| bundle.transport);
        rows.push(AgentRow {
            slug: slug.to_string(),
            description: format!("Generic {} agent", harness_name(*harness)),
            harness: *harness,
            bundle,
            transport,
            profile: None,
            per_session_key: None,
            kind: AgentKind::Generic,
            native_profile: None,
        });
    }
    rows.sort_by(|left, right| {
        left.slug
            .cmp(&right.slug)
            .then_with(|| left.harness.as_str().cmp(right.harness.as_str()))
    });
    rows
}

fn preferred_bundle(
    config: &HarnessesConfig,
    harness: Harness,
    native_profile: bool,
) -> Option<String> {
    let transports = match harness {
        Harness::Codex => [Transport::AppServer, Transport::Pty],
        Harness::ClaudeCode | Harness::Opencode => [Transport::Acp, Transport::Pty],
        Harness::Grok | Harness::Unknown => [Transport::Pty, Transport::Pty],
    };
    transports.into_iter().find_map(|transport| {
        config
            .bundles
            .iter()
            .find(|(_, bundle)| {
                bundle.harness == harness
                    && bundle.transport == transport
                    && crate::harness::driver::lookup(harness, transport).is_some()
                    && (!native_profile
                        || crate::harness::supports_native_agent(harness, transport))
            })
            .map(|(name, _)| name.clone())
    })
}

pub(super) fn harness_name(harness: Harness) -> &'static str {
    match harness {
        Harness::ClaudeCode => "Claude",
        Harness::Codex => "Codex",
        Harness::Opencode => "OpenCode",
        Harness::Grok => "Grok",
        Harness::Unknown => "Unknown",
    }
}

#[cfg(test)]
#[path = "data/tests.rs"]
mod tests;
