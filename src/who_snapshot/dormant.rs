use super::{scope, StoreReader, WhoRow, WhoSource};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub(super) fn push_retained_rows(
    store: StoreReader<'_>,
    current_root: Option<&str>,
    now: u64,
    local_host: &str,
    rows: &mut Vec<WhoRow>,
    other_agents: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<()> {
    let live_pubkeys: HashSet<String> = rows.iter().map(|row| row.pubkey.clone()).collect();
    for standing in store.list_retained_session_standing(now)? {
        if live_pubkeys.contains(&standing.pubkey)
            || scope::is_archived_channel(store, &standing.channel_h)
        {
            continue;
        }
        let Some(session) = store.get_session(&standing.pubkey)? else {
            continue;
        };
        let slug = store
            .session_identity(&standing.pubkey)
            .ok()
            .flatten()
            .map(|identity| identity.display_slug())
            .or_else(|| {
                store
                    .resolve_slug_for_pubkey(&standing.pubkey)
                    .ok()
                    .flatten()
            })
            .unwrap_or_else(|| session.agent_slug.clone());
        if current_root
            .map(|root| scope::scope_contains_channel(store, root, &standing.channel_h))
            .unwrap_or(true)
        {
            rows.push(retained_row(
                store,
                &session,
                &standing.channel_h,
                slug,
                local_host,
                now,
            ));
        } else if scope::is_root_channel(store, &standing.channel_h) {
            other_agents
                .entry(standing.channel_h)
                .or_default()
                .insert(slug);
        }
    }
    Ok(())
}

fn retained_row(
    store: StoreReader<'_>,
    session: &crate::state::Session,
    channel: &str,
    slug: String,
    local_host: &str,
    now: u64,
) -> WhoRow {
    let work_root = scope::work_root_for(store, channel);
    WhoRow {
        source: WhoSource::Local,
        state: crate::session_state::SessionState::Offline,
        slug,
        channel: channel.to_string(),
        status: session.title.clone(),
        activity: String::new(),
        dormant: true,
        host: local_host.to_string(),
        age_secs: Some(now.saturating_sub(session.stopped_at)),
        rel_cwd: String::new(),
        remote: false,
        work_root_display: work_root.clone(),
        work_root,
        pubkey: session.pubkey.clone(),
    }
}
