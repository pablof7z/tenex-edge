use super::{bool_at, int_at, str_at};
use serde_json::Value;

#[derive(Default)]
pub(super) struct RelayContext {
    pub(super) tags_valid: bool,
    pub(super) tag_count: i64,
    pub(super) channel_found: bool,
    pub(super) channel_name: String,
    pub(super) author_profile_found: bool,
    pub(super) author_slug: String,
    pub(super) membership_snapshot: bool,
    pub(super) author_role: String,
}

pub(super) fn context(
    store: &crate::state::Store,
    event: &crate::state::RelayEvent,
) -> anyhow::Result<RelayContext> {
    let tag_count = serde_json::from_str::<Value>(&event.tags_json)
        .ok()
        .and_then(|value| value.as_array().map(|tags| tags.len() as i64));
    let channel = if event.channel_h.is_empty() {
        None
    } else {
        store.get_channel(&event.channel_h)?
    };
    let profile = store.get_profile(&event.pubkey)?;
    let membership_snapshot = if event.channel_h.is_empty() {
        false
    } else {
        store.has_channel_membership_snapshot(&event.channel_h)?
    };
    let author_role = if event.channel_h.is_empty() {
        String::new()
    } else {
        store
            .list_channel_members(&event.channel_h)?
            .into_iter()
            .find(|member| member.pubkey == event.pubkey)
            .map(|member| member.role)
            .unwrap_or_default()
    };
    Ok(RelayContext {
        tags_valid: tag_count.is_some(),
        tag_count: tag_count.unwrap_or(-1),
        channel_found: channel.is_some(),
        channel_name: channel
            .as_ref()
            .map(|row| row.name.clone())
            .unwrap_or_default(),
        author_profile_found: profile.is_some(),
        author_slug: profile
            .as_ref()
            .map(|row| row.slug.clone())
            .unwrap_or_default(),
        membership_snapshot,
        author_role,
    })
}

pub(super) fn validation_reason(
    event: &crate::state::RelayEvent,
    context: &RelayContext,
) -> &'static str {
    if !context.tags_valid {
        return "relay event tags_json is not valid JSON array data";
    }
    if event.kind == crate::fabric::nip29::wire::KIND_CHAT as u32
        && !event.channel_h.is_empty()
        && context.membership_snapshot
        && context.author_role.is_empty()
    {
        return "hydrated channel membership snapshot does not include relay event author";
    }
    ""
}

pub(super) fn push_limitations(limitations: &mut Vec<String>, evidence: &Value) {
    let relay_channel = str_at(evidence, "relay_channel_h");
    if !relay_channel.is_empty() && !bool_at(evidence, "relay_channel_found") {
        limitations.push("relay event channel metadata is not materialized".to_string());
    }
    if !bool_at(evidence, "relay_author_profile_found") {
        limitations.push("relay event author profile is not materialized".to_string());
    }
    if int_at(evidence, "relay_kind") == crate::fabric::nip29::wire::KIND_CHAT as i64
        && !relay_channel.is_empty()
        && !bool_at(evidence, "relay_membership_snapshot")
    {
        limitations.push(
            "relay event author membership cannot be proven until channel roster snapshots hydrate"
                .to_string(),
        );
    }
}
