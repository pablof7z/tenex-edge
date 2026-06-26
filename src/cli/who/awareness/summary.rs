use crate::session::{DeltaKind, StatusDeltaItem};
use crate::state::{ChatLogRow, Store};
use crate::util::{pubkey_short, relative_time};
use std::collections::BTreeSet;

const ACTIVITY_LIMIT: u64 = 5;

pub(super) fn project_line(store: &Store, project: &str) -> String {
    store
        .get_project_meta(project)
        .ok()
        .flatten()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|about| format!("{project} -- {about}"))
        .unwrap_or_else(|| project.to_string())
}

pub(super) fn breadcrumb_line(store: &Store, breadcrumb: &[(String, String)]) -> String {
    breadcrumb
        .iter()
        .map(|(id, _)| titled_channel_ref(store, id))
        .collect::<Vec<_>>()
        .join(" > ")
}

pub(super) fn channel_summary_line(store: &Store, id: &str, now: u64) -> String {
    let count = channel_member_count(store, id, now);
    format!(
        "{} [{}]",
        titled_channel_ref(store, id),
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
    let mut members = store.list_group_members(project).unwrap_or_default();
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
        .filter(|(pubkey, _)| !store.is_backend_profile(pubkey))
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
    exclude_session: Option<&str>,
) -> Vec<StatusDeltaItem> {
    let subs = store.subchannels_of(project).unwrap_or_default();
    let mut channels = vec![project.to_string()];
    channels.extend(subs.iter().map(|(id, _, _)| id.clone()));
    store
        .status_delta_since_in(&channels, since, now, exclude_session)
        .unwrap_or_default()
        .into_iter()
        .filter(|item| item.kind != DeltaKind::Gone && item.derived.liveness.is_live())
        .collect()
}

pub(super) fn changed_member_lines(project: &str, items: &[StatusDeltaItem]) -> Vec<String> {
    items
        .iter()
        .filter(|item| item.snapshot.project == project)
        .filter_map(|item| useful_work_text(item).map(|status| (item, status)))
        .map(|(item, status)| format!("@{} - {status}", item.snapshot.agent_slug))
        .collect()
}

pub(super) fn changed_subchannel_lines(
    store: &Store,
    since: u64,
    project: &str,
    now: u64,
    subs: &[(String, String, usize)],
    items: &[StatusDeltaItem],
) -> Vec<String> {
    let active: BTreeSet<String> = store
        .semantic_active_channels_since(since, &[project.to_string()])
        .unwrap_or_default()
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    let changed: BTreeSet<String> = items
        .iter()
        .map(|item| item.snapshot.project.clone())
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
    store
        .semantic_active_channels_since(cutoff, &exclude)
        .unwrap_or_default()
        .into_iter()
        .take(5)
        .map(|(id, _)| channel_summary_line(store, &id, now))
        .collect()
}

pub(super) fn current_activity_lines(
    store: &Store,
    project: &str,
    since: u64,
    now: u64,
    exclude_session: Option<&str>,
) -> Vec<String> {
    store
        .list_chat_messages(project, since, None, 0, false)
        .unwrap_or_default()
        .into_iter()
        .filter(|row| exclude_session != Some(row.from_session.as_str()))
        .take(ACTIVITY_LIMIT as usize)
        .map(|row| activity_line(store, row, now))
        .collect()
}

fn titled_channel_ref(store: &Store, id: &str) -> String {
    let base = channel_ref(id);
    match known_channel_title(store, id) {
        Some(title) if title != id => format!("{base} -- {title}"),
        _ => base,
    }
}

fn known_channel_title(store: &Store, id: &str) -> Option<String> {
    match store
        .channel_title(id)
        .ok()
        .flatten()
        .or_else(|| store.latest_channel_work_title(id).ok().flatten())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(title) if title != id => Some(title),
        _ => store
            .latest_channel_work_title(id)
            .ok()
            .flatten()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != id),
    }
}

fn channel_member_count(store: &Store, id: &str, now: u64) -> usize {
    let members = store.list_group_members(id).unwrap_or_default();
    let n = members
        .iter()
        .filter(|(pubkey, _)| !store.is_backend_profile(pubkey))
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

fn useful_work_text(item: &StatusDeltaItem) -> Option<String> {
    if item.derived.title.is_empty() && item.derived.activity.is_empty() {
        return None;
    }
    Some(super::super::render::status_plain(
        &item.derived.title,
        &item.derived.activity,
        item.derived.busy,
    ))
}

fn activity_line(store: &Store, row: ChatLogRow, now: u64) -> String {
    let from = if row.from_slug.is_empty() {
        slug_for_pubkey(store, &row.from_pubkey)
    } else {
        row.from_slug
    };
    format!(
        "[@{from}, {}] {}",
        relative_time(row.created_at, now),
        row.body
    )
}

fn slug_for_pubkey(store: &Store, pubkey: &str) -> String {
    store
        .resolve_slug_for_pubkey(pubkey)
        .ok()
        .flatten()
        .unwrap_or_else(|| pubkey_short(pubkey))
}
