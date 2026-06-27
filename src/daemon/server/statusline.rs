use super::*;

pub(in crate::daemon::server) const STATUSLINE_RECENT_SECS: u64 = 30;
/// How long a distillation error stays visible in the statusline before expiring.
pub(in crate::daemon::server) const DISTILL_ERROR_TTL_SECS: u64 = 300;

#[derive(serde::Deserialize, Default)]
pub(in crate::daemon::server) struct StatuslineParams {
    #[serde(default)]
    pub(in crate::daemon::server) session: Option<String>,
    #[serde(default)]
    pub(in crate::daemon::server) env_session: Option<String>,
    #[serde(default)]
    pub(in crate::daemon::server) cwd: Option<String>,
    #[serde(default)]
    pub(in crate::daemon::server) agent: Option<String>,
}

/// `statusline`: everything the host's status bar renders, in one pure-read RPC.
/// Like `turn_check`, this is called constantly by the harness, so it must
/// NEVER write to state.db (no drains, no touches) — peeks only.
pub(in crate::daemon::server) fn rpc_statusline(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: StatuslineParams = serde_json::from_value(params.clone()).unwrap_or_default();
    // Session ID is the only locator needed. Fail open (empty bar) when it is
    // absent or stale — the next session_start reassert will refresh @te_session
    // on the tmux session and the bar recovers on the next status-interval tick.
    let session_id = p.session.as_deref().filter(|s| !s.is_empty());
    let rec = match session_id {
        Some(id) => match state.with_store(|s| s.get_session(id)).ok().flatten() {
            Some(r) => r,
            None => {
                // ID present but not in the DB — stale @te_session (e.g. after a
                // DB restore). Return a visible error instead of an empty bar.
                return Ok(serde_json::json!({ "error": "stale" }));
            }
        },
        None => return Ok(serde_json::json!({})),
    };
    let now = now_secs();
    let host = state.host.clone();
    // Routing scope (channel when set, else the per-session room) — the member
    // count and is_member check key on it so a `channels switch` is reflected in
    // the statusline without restarting.
    let scope = rec.route_scope().to_string();
    state.with_store(|s| {
        let member_count = s.count_group_members(&scope).unwrap_or(0);
        let is_member = s.is_group_member(&scope, &rec.agent_pubkey).unwrap_or(true);
        // Read busy + title + live activity from the canonical aggregate via
        // the SHARED projection (derive_status), so the statusline agrees with
        // `who`/turn-deltas. Pure read: no drains, no touches. The statusline
        // shows the activity line (the live "doing now" signal from kind:30315),
        // not the persistent title (the channel title segment carries that).
        let (working, title, activity) = s
            .local_session_snapshot(&rec.session_id)
            .ok()
            .flatten()
            .map(|snap| {
                let d = derive_status(&snap, now);
                (d.busy, d.title, d.activity)
            })
            .unwrap_or((false, String::new(), String::new()));
        // `channel_title` is the display name of the routing scope (channel or
        // per-session room) from the relay-authored kind:39000 metadata cache
        // (== the channel's title on the relay == what the relay renders as the
        // room's name). Falls back to the distilled session title when the
        // session is in its own per-session room (issue #6: the room is renamed
        // to the distilled title via kind:9002 edit-metadata; the local cache
        // may lag by one refresh).
        // Session's distilled title always wins when available — it's more
        // up-to-date than the relay-authored channel name.  Fall back to the
        // channel's display name only when no title has been produced yet.
        let channel_title = if !title.is_empty() {
            title.clone()
        } else {
            s.group_display_name(&scope).unwrap_or_default()
        };
        // `work_root` is the parent project a per-session room or task channel
        // hangs under, or the project itself for an ordinary project session.
        // This is the "Project" line in `who`, surfaced as `project-name`.
        let work_root = s
            .work_root_for_scope(&scope)
            .unwrap_or_else(|_| rec.project.clone());
        let pending_chat = s.peek_chat_mentions(&rec.session_id).unwrap_or_default();
        let recent_since = now.saturating_sub(STATUSLINE_RECENT_SECS);
        let recent_chat = s
            .list_recently_delivered_chat_mentions(&rec.session_id, recent_since)
            .unwrap_or_default();
        let mut pending_json = chat_rows_to_json(&pending_chat);
        sort_message_json(&mut pending_json);
        let mut recent_json = chat_rows_to_json(&recent_chat);
        sort_message_json(&mut recent_json);
        let distill_error = s
            .get_recent_session_error(&rec.session_id, now.saturating_sub(DISTILL_ERROR_TTL_SECS))
            .ok()
            .flatten();
        Ok(serde_json::json!({
            "agent": rec.agent_slug,
            "host": host,
            "session_id": rec.session_id,
            "work_root": work_root,
            "channel": scope,
            "channel_title": channel_title,
            "member_count": member_count,
            "is_member": is_member,
            "working": working,
            "title": title,
            "activity": activity,
            "pending": pending_json,
            "recent": recent_json,
            "distill_error": distill_error,
        }))
    })
}
