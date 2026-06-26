use crate::state::Store;
use std::fmt::Write as _;

mod summary;

use summary::{
    breadcrumb_line, changed_member_lines, changed_status_items, changed_subchannel_lines,
    channel_ref, channel_summary_line, current_activity_lines, member_lines,
    other_active_channel_lines, project_line,
};

const RECENT_SEMANTIC_WINDOW_SECS: u64 = 10 * 60;

pub(crate) fn render_awareness_snapshot(
    store: &Store,
    project: &str,
    now: u64,
    self_slug: &str,
    self_pubkey: &str,
) -> Option<String> {
    let breadcrumb = store.channel_breadcrumb(project).ok()?;
    if breadcrumb.is_empty() {
        return None;
    }

    let mut out = String::from("[tenex-edge] Fabric context\n\n");
    let _ = writeln!(out, "Project: {}", project_line(store, &breadcrumb[0].0));
    let _ = writeln!(out, "Channel: {}", breadcrumb_line(store, &breadcrumb));

    let members = member_lines(store, project, now, self_slug, self_pubkey);
    write_section(&mut out, "Members:", &members);

    let subs = store.subchannels_of(project).unwrap_or_default();
    if !subs.is_empty() {
        let lines = subs
            .iter()
            .map(|(id, _, _)| channel_summary_line(store, id, now))
            .collect::<Vec<_>>();
        write_section(&mut out, "Subchannels:", &lines);
    }

    let other = other_active_channel_lines(
        store,
        project,
        &subs,
        now.saturating_sub(RECENT_SEMANTIC_WINDOW_SECS),
        now,
    );
    write_section(&mut out, "Other active channels, last 10m:", &other);

    Some(out.trim_end().to_string())
}

pub(crate) fn render_awareness_update_since_turn(
    store: &Store,
    since: u64,
    project: &str,
    now: u64,
    exclude_session: Option<&str>,
) -> Option<String> {
    render_awareness_update(store, since, project, now, exclude_session, "last turn")
}

pub(crate) fn render_awareness_update_since_check(
    store: &Store,
    since: u64,
    project: &str,
    now: u64,
    exclude_session: Option<&str>,
) -> Option<String> {
    render_awareness_update(store, since, project, now, exclude_session, "last check")
}

fn render_awareness_update(
    store: &Store,
    since: u64,
    project: &str,
    now: u64,
    exclude_session: Option<&str>,
    label: &str,
) -> Option<String> {
    let subs = store.subchannels_of(project).unwrap_or_default();
    let changed = changed_status_items(store, since, project, now, exclude_session);

    let members = changed_member_lines(project, &changed);
    let subchannels = changed_subchannel_lines(store, since, project, now, &subs, &changed);
    let other = other_active_channel_lines(
        store,
        project,
        &subs,
        since.max(now.saturating_sub(RECENT_SEMANTIC_WINDOW_SECS)),
        now,
    );
    let activity = current_activity_lines(store, project, since, now, exclude_session);

    if members.is_empty() && subchannels.is_empty() && other.is_empty() && activity.is_empty() {
        return None;
    }

    let mut out = format!("[tenex-edge] Fabric updates since your {label}");
    write_section(&mut out, "Members:", &members);
    write_section(&mut out, "Subchannels:", &subchannels);
    write_section(&mut out, "Other active channels, last 10m:", &other);
    if !activity.is_empty() {
        write_section(
            &mut out,
            &format!("Activity in {}:", channel_ref(project)),
            &activity,
        );
    }
    Some(out.trim_end().to_string())
}

fn write_section(out: &mut String, title: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    let _ = writeln!(out);
    let _ = writeln!(out);
    let _ = writeln!(out, "{title}");
    for line in lines {
        let _ = writeln!(out, "- {line}");
    }
}

#[cfg(test)]
mod tests;
