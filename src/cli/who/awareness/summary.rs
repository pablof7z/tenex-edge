use crate::session::DerivedStatus;
use crate::state::{RelayEvent, Store};
use crate::util::{pubkey_short, relative_time};
use std::collections::BTreeSet;

const ACTIVITY_LIMIT: u32 = 5;
/// Max parent links to walk when building a channel breadcrumb (cycle guard).
const MAX_BREADCRUMB_DEPTH: usize = 16;

/// One agent whose live status changed (updated_at > since) within a channel.
/// Replaces the old cross-module `StatusDeltaItem`: the awareness layer only
/// needs the channel, the agent slug, and the derived view.
pub(super) struct StatusChange {
    pub(super) channel_h: String,
    pub(super) slug: String,
    pub(super) derived: DerivedStatus,
}

// ── channel topology (reconstructed from relay_channels primitives) ───────────

/// Breadcrumb from the root project down to `project`, as `(channel_h, name)`
/// pairs (root first). Empty when the channel is not yet materialized — the
/// caller treats that as "no fabric context to show".
pub(super) fn channel_breadcrumb(store: &Store, project: &str) -> Vec<(String, String)> {
    if store.get_channel(project).ok().flatten().is_none() {
        return Vec::new();
    }
    let mut chain = Vec::new();
    let mut cur = project.to_string();
    for _ in 0..MAX_BREADCRUMB_DEPTH {
        let name = store
            .get_channel(&cur)
            .ok()
            .flatten()
            .map(|c| c.name)
            .unwrap_or_default();
        chain.push((cur.clone(), name));
        match store.channel_parent(&cur).ok().flatten() {
            Some(parent) if !parent.is_empty() => cur = parent,
            _ => break,
        }
    }
    chain.reverse();
    chain
}

/// Direct child channels of `project` as `(channel_h, name, member_count)`.
pub(super) fn subchannels_of(store: &Store, project: &str) -> Vec<(String, String, usize)> {
    store
        .list_channels()
        .unwrap_or_default()
        .into_iter()
        .filter(|c| c.parent == project)
        .map(|c| {
            let count = store.count_channel_members(&c.channel_h).unwrap_or(0) as usize;
            (c.channel_h, c.name, count)
        })
        .collect()
}

// ── line builders ─────────────────────────────────────────────────────────────

pub(super) fn project_line(store: &Store, project: &str) -> String {
    channel_about(store, project)
        .filter(|s| !s.is_empty())
        .map(|about| format!("{project} -- {about}"))
        .unwrap_or_else(|| project.to_string())
}

pub(super) fn breadcrumb_line(store: &Store, breadcrumb: &[(String, String)], now: u64) -> String {
    breadcrumb
        .iter()
        .map(|(id, _)| titled_channel_ref(store, id, now))
        .collect::<Vec<_>>()
        .join(" > ")
}

pub(super) fn channel_summary_line(store: &Store, id: &str, now: u64) -> String {
    let count = channel_member_count(store, id, now);
    format!(
        "{} [{}]",
        titled_channel_ref(store, id, now),
        member_count_label(count)
    )
}

pub(super) fn channel_ref(id: &str) -> String {
    if id.starts_with('#') {
        id.to_string()
    } else {
        format!("#{id}")
    }
}

pub(super) fn member_lines(
    store: &Store,
    project: &str,
    now: u64,
    self_slug: &str,
    self_pubkey: &str,
) -> Vec<String> {
    let status_map = super::super::channel::channel_status_map(store, project, now);
    let mut members: Vec<(String, String)> = store
        .list_channel_members(project)
        .unwrap_or_default()
        .into_iter()
        .map(|m| (m.pubkey, m.role))
        .collect();
    let mut seen = members
        .iter()
        .map(|(pubkey, _)| pubkey.clone())
        .collect::<BTreeSet<_>>();
    if !members.iter().any(|(pk, _)| pk == self_pubkey) {
        members.push((self_pubkey.to_string(), "member".to_string()));
        seen.insert(self_pubkey.to_string());
    }
    for pubkey in status_map.keys().collect::<BTreeSet<_>>() {
        if !seen.contains(pubkey.as_str()) {
            members.push((pubkey.to_string(), "member".to_string()));
        }
    }
    members
        .into_iter()
        .filter(|(pubkey, _)| !is_backend(store, pubkey))
        .map(|(pubkey, role)| {
            let slug = if pubkey == self_pubkey {
                self_slug.to_string()
            } else {
                slug_for_pubkey(store, &pubkey)
            };
            let you = if pubkey == self_pubkey { " (you)" } else { "" };
            let status = status_map
                .get(&pubkey)
                .map(|s| super::super::render::status_plain(&s.title, &s.activity, s.busy))
                .unwrap_or_else(|| offline_label(&role));
            format!("@{slug}{you} - {status}")
        })
        .collect()
}

pub(super) fn changed_status_items(
    store: &Store,
    since: u64,
    project: &str,
    now: u64,
    exclude_pubkey: Option<&str>,
) -> Vec<StatusChange> {
    let subs = subchannels_of(store, project);
    let mut channels = vec![project.to_string()];
    channels.extend(subs.iter().map(|(id, _, _)| id.clone()));
    let mut out = Vec::new();
    for ch in &channels {
        for st in store.live_status_for_channel(ch, now).unwrap_or_default() {
            // The viewer's own status (same signing pubkey) is never echoed back.
            if exclude_pubkey == Some(st.pubkey.as_str()) {
                continue;
            }
            // Only rows that actually changed since the cursor are "new".
            if st.updated_at <= since {
                continue;
            }
            out.push(StatusChange {
                channel_h: ch.clone(),
                slug: peer_slug(store, &st),
                derived: super::super::channel::derive_from_status(&st, now),
            });
        }
    }
    out
}

pub(super) fn changed_member_lines(project: &str, items: &[StatusChange]) -> Vec<String> {
    items
        .iter()
        .filter(|item| item.channel_h == project)
        .filter_map(|item| useful_work_text(item).map(|status| (item, status)))
        .map(|(item, status)| format!("@{} - {status}", item.slug))
        .collect()
}

pub(super) fn changed_subchannel_lines(
    store: &Store,
    since: u64,
    project: &str,
    now: u64,
    subs: &[(String, String, usize)],
    items: &[StatusChange],
) -> Vec<String> {
    let active: BTreeSet<String> = active_channels_since(store, since, &[project.to_string()]);
    let changed: BTreeSet<String> = items
        .iter()
        .map(|item| item.channel_h.clone())
        .filter(|id| id != project)
        .collect();
    subs.iter()
        .map(|(id, _, _)| id)
        .filter(|id| active.contains(*id) || changed.contains(*id))
        .map(|id| channel_summary_line(store, id, now))
        .collect()
}

pub(super) fn other_active_channel_lines(
    store: &Store,
    project: &str,
    subs: &[(String, String, usize)],
    cutoff: u64,
    now: u64,
) -> Vec<String> {
    let mut exclude = vec![project.to_string()];
    exclude.extend(subs.iter().map(|(id, _, _)| id.clone()));
    let mut channels: Vec<String> = active_channels_since(store, cutoff, &exclude)
        .into_iter()
        .collect();
    channels.sort();
    channels
        .into_iter()
        .take(5)
        .map(|id| channel_summary_line(store, &id, now))
        .collect()
}

pub(super) fn current_activity_lines(
    store: &Store,
    project: &str,
    since: u64,
    now: u64,
    exclude_pubkey: Option<&str>,
) -> Vec<String> {
    store
        .chat_for_channel(project, since, ACTIVITY_LIMIT)
        .unwrap_or_default()
        .into_iter()
        .filter(|row| exclude_pubkey != Some(row.pubkey.as_str()))
        .map(|row| activity_line(store, row, now))
        .collect()
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Channels (other than `exclude`) whose status changed since `cutoff`.
fn active_channels_since(store: &Store, cutoff: u64, exclude: &[String]) -> BTreeSet<String> {
    let excl: BTreeSet<&str> = exclude.iter().map(String::as_str).collect();
    store
        .active_channels_since(cutoff)
        .unwrap_or_default()
        .into_iter()
        .filter(|id| !excl.contains(id.as_str()))
        .collect()
}

fn channel_about(store: &Store, id: &str) -> Option<String> {
    store
        .get_channel(id)
        .ok()
        .flatten()
        .map(|c| c.about.trim().to_string())
}

fn titled_channel_ref(store: &Store, id: &str, now: u64) -> String {
    let base = channel_ref(id);
    match known_channel_title(store, id, now) {
        Some(title) if title != id => format!("{base} -- {title}"),
        _ => base,
    }
}

/// Title for a channel reference. A real kind:39000 `name` wins; when the channel
/// exists but carries no meaningful name (empty or == its own id, as session
/// rooms do), fall back to the live work title of whoever is active there. No
/// channel record at all → no title (avoids labelling transient/transport
/// channels we have never materialized).
fn known_channel_title(store: &Store, id: &str, now: u64) -> Option<String> {
    let channel = store.get_channel(id).ok().flatten()?;
    let name = channel.name.trim().to_string();
    if !name.is_empty() && name != id {
        return Some(name);
    }
    latest_channel_work_title(store, id, now)
}

/// Most-recently-updated live status title in a channel (the agent's current work
/// text), used as the channel's display title when it has no proper name.
fn latest_channel_work_title(store: &Store, id: &str, now: u64) -> Option<String> {
    store
        .live_status_for_channel(id, now)
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.title.trim().to_string())
        .find(|t| !t.is_empty() && t != id)
}

fn channel_member_count(store: &Store, id: &str, now: u64) -> usize {
    let n = store
        .list_channel_members(id)
        .unwrap_or_default()
        .into_iter()
        .filter(|m| !is_backend(store, &m.pubkey))
        .count();
    if n > 0 {
        return n;
    }
    super::super::channel::channel_status_map(store, id, now)
        .values()
        .filter(|status| status.liveness.is_live())
        .count()
}

fn member_count_label(count: usize) -> String {
    match count {
        1 => "1 member".to_string(),
        n => format!("{n} members"),
    }
}

fn offline_label(role: &str) -> String {
    if role == "admin" {
        "Human".to_string()
    } else {
        "offline".to_string()
    }
}

fn useful_work_text(item: &StatusChange) -> Option<String> {
    if item.derived.title.is_empty() && item.derived.activity.is_empty() {
        return None;
    }
    Some(super::super::render::status_plain(
        &item.derived.title,
        &item.derived.activity,
        item.derived.busy,
    ))
}

fn activity_line(store: &Store, row: RelayEvent, now: u64) -> String {
    let from = slug_for_pubkey(store, &row.pubkey);
    format!(
        "[@{from}, {}] {}",
        relative_time(row.created_at, now),
        row.content
    )
}

fn is_backend(store: &Store, pubkey: &str) -> bool {
    store
        .get_profile(pubkey)
        .ok()
        .flatten()
        .map(|p| p.is_backend)
        .unwrap_or(false)
}

fn peer_slug(store: &Store, st: &crate::state::Status) -> String {
    if !st.slug.is_empty() {
        return st.slug.clone();
    }
    slug_for_pubkey(store, &st.pubkey)
}

fn slug_for_pubkey(store: &Store, pubkey: &str) -> String {
    store
        .resolve_slug_for_pubkey(pubkey)
        .ok()
        .flatten()
        .unwrap_or_else(|| pubkey_short(pubkey))
}
