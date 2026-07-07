use super::model::MessageRow;
use super::refs::{display_name, pubkey_ref};
use super::{FabricContextInput, FabricMessageSeed};
use crate::state::{RelayEvent, Store};
use crate::util::{relative_time, truncate_words, CHAT_RENDER_WORD_LIMIT};
use std::collections::HashSet;

const WINDOW_SECS: u64 = 4 * 60 * 60;
const CHAT_CAP: u32 = 10_000;
const MAX_CLUSTER_GAP_SECS: u64 = 20 * 60;
const MAX_CLUSTER_ROWS: usize = 30;

pub(super) fn message_rows(
    store: &Store,
    input: &FabricContextInput<'_>,
    channel: &str,
    forced: &[FabricMessageSeed],
) -> (Vec<MessageRow>, usize) {
    let since = if input.cursor == 0 {
        input.now.saturating_sub(WINDOW_SECS)
    } else {
        input.cursor
    };
    let mut events = store
        .chat_for_channel(channel, since, CHAT_CAP)
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.kind == crate::fabric::nip29::wire::KIND_CHAT as u32)
        .filter(|e| e.pubkey != input.self_pubkey)
        .collect::<Vec<_>>();
    // Messages already pasted verbatim into the pane (e.g. the mention that
    // spawned this turn) would otherwise also show up here, duplicating the
    // same text the agent already saw as literal prompt input.
    if let Some(session) = input.session {
        let injected: HashSet<String> = store
            .injected_for_session(&session.session_id)
            .unwrap_or_default()
            .into_iter()
            .map(|row| row.event_id)
            .collect();
        if !injected.is_empty() {
            events.retain(|e| !injected.contains(&e.id));
        }
    }
    let omitted = if input.cursor == 0 {
        let total = events.len();
        events = recent_cluster(events);
        total.saturating_sub(events.len())
    } else {
        0
    };
    let mut seen: HashSet<String> = events.iter().map(|e| e.id.clone()).collect();
    for row in forced {
        if seen.insert(row.id.clone()) {
            events.push(RelayEvent {
                id: row.id.clone(),
                kind: crate::fabric::nip29::wire::KIND_CHAT as u32,
                pubkey: row.from_pubkey.clone(),
                created_at: row.created_at,
                channel_h: channel.to_string(),
                d_tag: String::new(),
                content: row.body.clone(),
                tags_json: "[]".to_string(),
            });
        }
    }
    events.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
    let rows = events
        .into_iter()
        .map(|ev| {
            let mention = forced.iter().any(|f| f.id == ev.id && f.mention)
                || mentions_pubkey(&ev.tags_json, input.self_pubkey);
            message_row(store, &ev, input.now, input.local_host, mention)
        })
        .collect();
    (rows, omitted)
}

fn recent_cluster(mut events: Vec<RelayEvent>) -> Vec<RelayEvent> {
    if events.len() <= MAX_CLUSTER_ROWS {
        return events;
    }
    events.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
    let mut start = events.len().saturating_sub(1);
    while start > 0 {
        let gap = events[start]
            .created_at
            .saturating_sub(events[start - 1].created_at);
        if gap > MAX_CLUSTER_GAP_SECS || events.len() - start >= MAX_CLUSTER_ROWS {
            break;
        }
        start -= 1;
    }
    events.split_off(start)
}

fn message_row(
    store: &Store,
    ev: &RelayEvent,
    now: u64,
    local_host: &str,
    mention: bool,
) -> MessageRow {
    let (body, truncated) = truncate_words(&ev.content, CHAT_RENDER_WORD_LIMIT);
    MessageRow {
        id: ev.id.clone(),
        channel: display_name(store, &ev.channel_h),
        from: pubkey_ref(store, &ev.pubkey, local_host),
        age: relative_time(ev.created_at, now),
        body,
        mention,
        truncated,
    }
}

pub(super) fn mentions_pubkey(tags_json: &str, pubkey: &str) -> bool {
    if pubkey.is_empty() {
        return false;
    }
    let Ok(tags) = serde_json::from_str::<Vec<Vec<String>>>(tags_json) else {
        return false;
    };
    tags.iter()
        .any(|tag| tag.first().is_some_and(|t| t == "p") && tag.get(1).is_some_and(|p| p == pubkey))
}
