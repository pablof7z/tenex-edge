use crate::domain::{AgentRef, DomainEvent, Profile};
use anyhow::Result;
use nostr_sdk::prelude::{Event, EventBuilder};

use super::{all_tag_values, first_tag, has_bare_tag, kind, tag, KIND_PROFILE};

pub(super) fn encode(pf: &Profile) -> Result<EventBuilder> {
    let name = if pf.is_backend {
        pf.agent.slug.clone()
    } else {
        crate::idref::agent_label(&pf.agent.slug, &pf.host)
    };
    let content = serde_json::json!({ "name": name }).to_string();
    let mut tags = vec![tag(&["host", &pf.host])?];
    if !pf.agent_slug.is_empty() {
        tags.push(tag(&["agent-slug", &pf.agent_slug])?);
    }
    for owner in &pf.owners {
        tags.push(tag(&["p", owner])?);
    }
    if pf.is_backend {
        tags.push(tag(&["backend"])?);
    }
    Ok(EventBuilder::new(kind(KIND_PROFILE), content)
        .tags(tags)
        .allow_self_tagging())
}

pub(super) fn decode(event: &Event, pubkey: String) -> Option<DomainEvent> {
    let host = first_tag(event, "host").unwrap_or_default().to_string();
    let is_backend = has_bare_tag(event, "backend");
    let name = name_from_metadata(&event.content);
    let slug = if is_backend {
        name
    } else {
        crate::idref::slug_from_profile_name(&name, &host)
    };
    Some(DomainEvent::Profile(Profile {
        agent: AgentRef::new(pubkey, slug),
        agent_slug: agent_slug(event),
        host,
        owners: all_tag_values(event, "p"),
        is_backend,
    }))
}

fn agent_slug(event: &Event) -> String {
    first_tag(event, "agent-slug")
        .or_else(|| first_tag(event, "agentSlug"))
        .unwrap_or_default()
        .to_string()
}

fn name_from_metadata(content: &str) -> String {
    serde_json::from_str::<serde_json::Value>(content)
        .ok()
        .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(String::from))
        .unwrap_or_default()
}
