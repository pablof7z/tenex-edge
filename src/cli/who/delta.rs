use super::*;

/// Append the turn-start "tenex-edge fabric" block(s): the full roster on the
/// first turn, or "changes since your last turn" afterward. This is the single
/// source of truth — both the CLI `turn_start` and the daemon's `turn_start` RPC
/// call it, so the injected text is identical.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_turn_fabric_block(
    store: &std::sync::Mutex<Store>,
    blocks: &mut Vec<String>,
    first_turn: bool,
    prev_turn_started_at: u64,
    project: &str,
    now: u64,
    daemon_host: &str,
    self_session: &str,
    self_slug: &str,
    self_pubkey: &str,
) {
    let store = store.lock().expect("store mutex poisoned");
    if first_turn {
        // The channel-hierarchy context: where the agent sits in the channel
        // tree, who shares its channel, the subchannels beneath, and a pointer
        // to the rest of the fabric.
        if let Some(block) =
            crate::cli::who::render_channel_context(&store, project, now, self_slug, self_pubkey)
        {
            blocks.push(block);
        }
    } else {
        // Self-exclude the viewer's own session: rpc_turn_start opens this turn
        // (busy transition) BEFORE context assembly, so without this the session
        // would see its own just-started change echoed back as a delta.
        let delta = build_status_delta(
            &store,
            prev_turn_started_at,
            project,
            now,
            daemon_host,
            Some(self_session),
        );
        if !delta.is_empty() {
            blocks.push(format!(
                "tenex-edge fabric — changes since your last turn:\n{}",
                delta.join("\n")
            ));
        }
    }
}

/// Build the "changes since X" delta lines from the single shared delta query.
/// Every in-scope session (local AND peer) is classified by `status_delta_since`
/// into exactly one of appeared / changed / gone since `since`, project-scoped,
/// with `exclude_session` (the viewer's own session) filtered out at the source.
/// Shared by the turn-start delta (subsequent turns) and the mid-turn PostToolUse
/// check, so both render identically.
///
/// - Appeared: a session that joined since the cursor (`● … joined`).
/// - Changed:  a versioned content change — the agent finished (busy→idle) or a
///   new title landed (`↻ … — <status>`).
/// - Gone:     the session ended/was superseded, or its liveness expired in the
///   window (`✗ … left`). A dropped-off session stays reportable as gone.
pub(crate) fn build_status_delta(
    store: &Store,
    since: u64,
    project: &str,
    now: u64,
    daemon_host: &str,
    exclude_session: Option<&str>,
) -> Vec<String> {
    // Scope the delta to the current channel AND its subtree, so an agent sees
    // activity in subchannels beneath it (a new subchannel's first agent, a
    // sibling working below) — not just its own channel. Each subchannel's
    // display label is kept to tag cross-channel deltas.
    let subs = store.subchannels_of(project).unwrap_or_default();
    let mut channels: Vec<String> = Vec::with_capacity(subs.len() + 1);
    channels.push(project.to_string());
    channels.extend(subs.iter().map(|(id, _, _)| id.clone()));
    let labels: std::collections::HashMap<String, String> =
        subs.into_iter().map(|(id, name, _)| (id, name)).collect();

    let items = store
        .status_delta_since_in(&channels, since, now, exclude_session)
        .unwrap_or_default();
    if items.is_empty() {
        return Vec::new();
    }

    let name_counts = delta_agent_name_counts(store, &items, project, now, daemon_host);

    // Canonical presence lines, one per change. master's name disambiguation
    // (delta_agent_label) is preserved; a delta from a subchannel additionally
    // gets a ` #<subchannel>` suffix so the agent knows where it happened.
    //   * bravo4217 (codex@laptop) joined
    //   * echo0163 (claude@tower) left #research
    //   * bravo4217 (codex@laptop) — reviewing the patch
    let mut delta: Vec<String> = Vec::with_capacity(items.len());
    for item in &items {
        let snap = &item.snapshot;
        let label = delta_agent_label(snap, &name_counts);
        let activity = super::render::status_plain("", &item.derived.activity, item.derived.busy);
        let suffix = if snap.project != project {
            let name = labels
                .get(snap.project.as_str())
                .cloned()
                .unwrap_or_else(|| snap.project.clone());
            format!(" #{name}")
        } else {
            String::new()
        };
        let line = match item.kind {
            DeltaKind::Appeared => format!("* {label} joined{suffix}"),
            DeltaKind::Gone => format!("* {label} left{suffix}"),
            DeltaKind::Changed => format!("* {label} — {activity}{suffix}"),
        };
        delta.push(line);
    }
    delta
}

fn delta_agent_label(
    snap: &SessionSnapshot,
    name_counts: &std::collections::BTreeMap<String, usize>,
) -> String {
    let agent = super::render::display_agent_name(
        snap.agent_slug.as_str(),
        snap.session_id.as_str(),
        name_counts,
    );
    let host = slugify_host(&snap.host);
    if host.is_empty() {
        agent
    } else {
        format!("{agent} ({host})")
    }
}

fn delta_agent_name_counts(
    store: &Store,
    items: &[StatusDeltaItem],
    project: &str,
    now: u64,
    daemon_host: &str,
) -> std::collections::BTreeMap<String, usize> {
    let mut seen = std::collections::BTreeSet::new();
    if let Ok(snapshot) = super::snapshot::load_who_snapshot(store, Some(project), now, daemon_host)
    {
        for row in snapshot.rows {
            seen.insert((row.slug, row.session_id));
        }
    }
    for item in items {
        let snap = &item.snapshot;
        if snap.project == project {
            seen.insert((
                snap.agent_slug.clone(),
                snap.session_id.as_str().to_string(),
            ));
        }
    }

    let mut counts = std::collections::BTreeMap::new();
    for (slug, _) in seen {
        *counts.entry(slug).or_insert(0) += 1;
    }
    counts
}
