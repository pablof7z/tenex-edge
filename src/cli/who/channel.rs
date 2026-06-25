use super::*;

pub(super) fn channel_status_map(
    store: &Store,
    channel: &str,
    now: u64,
) -> std::collections::HashMap<String, crate::session::DerivedStatus> {
    let since = now.saturating_sub(crate::session::STATUS_TTL_SECS);
    let mut map = std::collections::HashMap::new();
    // Peers first so a local session of the same agent overrides it.
    for snap in store
        .peer_session_snapshots(Some(channel), since)
        .unwrap_or_default()
    {
        map.insert(
            snap.agent_pubkey.clone(),
            crate::session::derive_status(&snap, now),
        );
    }
    for snap in store
        .live_session_snapshots(Some(channel), since)
        .unwrap_or_default()
    {
        let pubkey = store
            .session_pubkey_for_session(snap.session_id.as_str())
            .unwrap_or_else(|| snap.agent_pubkey.clone());
        map.insert(pubkey, crate::session::derive_status(&snap, now));
    }
    map
}

/// Count of distinct LIVE agents in `channel` and whether any is busy.
fn channel_agent_activity(store: &Store, channel: &str, now: u64) -> (usize, bool) {
    let map = channel_status_map(store, channel, now);
    let live: Vec<_> = map.values().filter(|ds| ds.liveness.is_live()).collect();
    let busy = live.iter().any(|ds| ds.busy);
    (live.len(), busy)
}

/// Render the channel-hierarchy context block injected at an agent's first turn.
/// Shows the agent's identity, where it sits in the channel tree, who else is in
/// the current channel, the subchannels beneath it, and a pointer to the rest of
/// the fabric. Returns `None` when there is no resolvable channel.
pub(crate) fn render_channel_context(
    store: &Store,
    project: &str,
    now: u64,
    self_slug: &str,
    self_pubkey: &str,
) -> Option<String> {
    use std::fmt::Write as _;

    let breadcrumb = store.channel_breadcrumb(project).ok()?;
    if breadcrumb.is_empty() {
        return None;
    }
    let my_pubkey = self_pubkey;
    let my_slug = self_slug;
    let root_label = &breadcrumb[0].1;
    let leaf_label = breadcrumb
        .last()
        .map(|(_, label)| label.clone())
        .unwrap_or_default();
    let crumb = breadcrumb
        .iter()
        .map(|(_, label)| format!("#{label}"))
        .collect::<Vec<_>>()
        .join(" > ");

    let mut out = String::new();
    // Intro names the agent and its current channel (no session code — codenames
    // are an internal handle, not the agent's identity); the hierarchy follows.
    let _ = writeln!(
        out,
        "[tenex-edge] You are {my_slug} on #{leaf_label} — the channel hierarchy is shown below."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Project: {root_label}");
    let _ = writeln!(out, "Channel: {crumb}");
    if let Some(about) = store
        .get_project_meta(project)
        .ok()
        .flatten()
        .filter(|a| !a.is_empty())
    {
        let _ = writeln!(out, "Description: {about}");
    }

    // Members of the current channel, with their live activity.
    let members = store.list_group_members(project).unwrap_or_default();
    if !members.is_empty() {
        let status_map = channel_status_map(store, project, now);
        let mut parts: Vec<String> = Vec::new();
        for (pubkey, role) in &members {
            if store.is_backend_profile(pubkey) {
                continue;
            }
            let you = if pubkey.as_str() == my_pubkey {
                " (you)"
            } else {
                ""
            };
            let label = match status_map.get(pubkey) {
                Some(ds) if ds.busy && !ds.activity.is_empty() => ds.activity.clone(),
                Some(_) => "idle".to_string(),
                // No live session: an admin with no agent identity reads as a
                // human; an agent member that is simply offline reads as such.
                None if role == "admin" => "Human".to_string(),
                None => "offline".to_string(),
            };
            let slug = store
                .resolve_slug_for_pubkey(pubkey)
                .ok()
                .flatten()
                .unwrap_or_else(|| crate::util::pubkey_short(pubkey));
            parts.push(format!("@{slug}{you} - {label}"));
        }
        let _ = writeln!(out, "Members: {}", parts.join(" / "));
    }

    // Subchannels beneath the current channel, indented by depth.
    let subs = store.subchannels_of(project).unwrap_or_default();
    if !subs.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "Subchannels:");
        for (id, name, depth) in &subs {
            let (count, busy) = channel_agent_activity(store, id, now);
            let indent = "  ".repeat(depth.saturating_sub(1));
            let agents = if count == 1 {
                "1 agent".to_string()
            } else {
                format!("{count} agents")
            };
            let status = if busy { "active" } else { "idle" };
            let _ = writeln!(out, "{indent}#{name} ({agents}) - {status}");
        }
    }

    // The rest of the fabric: how many other channels saw activity in 24h.
    let mut exclude: Vec<String> = vec![project.to_string()];
    exclude.extend(subs.iter().map(|(id, _, _)| id.clone()));
    let other = store
        .count_active_channels_since(now.saturating_sub(86_400), &exclude)
        .unwrap_or(0);
    if other > 0 {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "There {} {other} other active channel{} in the past 24 hours. \
             Use `tenex-edge channels list` for more.",
            if other == 1 { "is" } else { "are" },
            if other == 1 { "" } else { "s" }
        );
    }

    let _ = writeln!(out);
    let _ = write!(
        out,
        "To reach another agent, mention its `@name` (as listed above) in a \
         `tenex-edge chat write --message \"...\"` — run `tenex-edge chat write` whenever asked \
         to contact or notify another agent; do not say you cannot."
    );
    Some(out)
}
