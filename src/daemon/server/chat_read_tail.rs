use super::*;

#[derive(serde::Deserialize, Default)]
pub(in crate::daemon::server) struct ChatReadParams {
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    session: Option<String>,
    #[serde(default)]
    env_session: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    since: Option<u64>,
    #[serde(default)]
    limit: Option<u64>,
    #[serde(default)]
    offset: Option<u64>,
    #[serde(default)]
    tail: bool,
    #[serde(default)]
    live: bool,
}

pub(in crate::daemon::server) async fn handle_chat_read<W: AsyncWriteExt + Unpin>(
    state: &Arc<DaemonState>,
    id: u64,
    params: &serde_json::Value,
    writer: &mut W,
) -> Result<()> {
    let p: ChatReadParams = serde_json::from_value(params.clone()).unwrap_or_default();
    let scope = match p.channel.filter(|s| !s.trim().is_empty()) {
        Some(channel) => channel,
        None => resolve_session_inner(
            state,
            p.session.as_deref(),
            p.env_session.as_deref(),
            p.cwd.as_deref(),
            p.agent.as_deref(),
            p.group.as_deref(),
            false,
        )?
        .route_scope()
        .to_string(),
    };
    let since = p.since.unwrap_or(0);
    let offset = p.offset.unwrap_or(0);

    let _ = ensure_subscription(state, &scope).await;
    let mut rx = if p.live {
        Some(state.tail_subscribe())
    } else {
        None
    };
    let live_started_at = now_secs();

    let rows = state.with_store(|s| {
        let mut scopes = vec![scope.clone()];
        if s.session_room_parent(&scope).ok().flatten().is_none() {
            scopes.extend(s.session_rooms_under(&scope).unwrap_or_default());
        }
        let mut rows: Vec<ChatLogRow> = scopes
            .iter()
            .flat_map(|scope| {
                s.list_chat_messages(scope, since, None, 0, false)
                    .unwrap_or_default()
            })
            .collect();
        rows.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.chat_event_id.cmp(&b.chat_event_id))
        });
        if p.tail {
            let limit = p.limit.unwrap_or(10) as usize;
            let start = rows
                .len()
                .saturating_sub(limit.saturating_add(offset as usize));
            let end = rows.len().saturating_sub(offset as usize);
            rows = rows.get(start..end).unwrap_or(&[]).to_vec();
        } else {
            let start = offset as usize;
            let end = p
                .limit
                .map(|limit| start.saturating_add(limit as usize))
                .unwrap_or(rows.len())
                .min(rows.len());
            rows = rows.get(start..end).unwrap_or(&[]).to_vec();
        }
        rows
    });
    let mut seen: std::collections::HashSet<String> =
        rows.iter().map(|r| r.chat_event_id.clone()).collect();
    let mut cursor = rows
        .iter()
        .map(|r| r.created_at)
        .max()
        .unwrap_or(live_started_at.max(since));

    for row in rows {
        if write_json(writer, &Response::item(id, chat_log_row_to_json(&row)))
            .await
            .is_err()
        {
            let _ = write_json(writer, &Response::end(id)).await;
            return Ok(());
        }
    }

    let Some(ref mut rx) = rx else {
        let _ = write_json(writer, &Response::end(id)).await;
        return Ok(());
    };

    loop {
        match rx.recv().await {
            Ok(TailEvent::Msg {
                project: ev_project,
                ..
            }) if ev_project == scope => {
                let rows = state.with_store(|s| {
                    s.list_chat_messages(&scope, cursor, None, 0, false)
                        .unwrap_or_default()
                });
                for row in rows {
                    if !seen.insert(row.chat_event_id.clone()) {
                        continue;
                    }
                    cursor = cursor.max(row.created_at);
                    if write_json(writer, &Response::item(id, chat_log_row_to_json(&row)))
                        .await
                        .is_err()
                    {
                        let _ = write_json(writer, &Response::end(id)).await;
                        return Ok(());
                    }
                }
            }
            Ok(_) => {}
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
        }
    }
    let _ = write_json(writer, &Response::end(id)).await;
    Ok(())
}

pub(in crate::daemon::server) fn chat_log_row_to_json(row: &ChatLogRow) -> serde_json::Value {
    serde_json::json!({
        "event_id": &row.chat_event_id,
        "from_pubkey": &row.from_pubkey,
        "from_slug": &row.from_slug,
        "host": &row.host,
        "project": &row.project,
        "body": &row.body,
        "created_at": row.created_at,
        "from_session": &row.from_session,
        "mentioned_session": &row.mentioned_session,
    })
}

// ── tail (streaming) ──────────────────────────────────────────────────────────

/// Parameters for the `tail` RPC.
#[derive(serde::Deserialize, Default)]
pub(in crate::daemon::server) struct TailParams {
    #[serde(default)]
    project: Option<String>,
    /// Number of backfill events (recent messages + roster snapshot), default 20.
    #[serde(default)]
    backfill: Option<u64>,
    /// Return only events after this unix timestamp.
    #[serde(default)]
    since: Option<u64>,
}

pub(in crate::daemon::server) async fn handle_tail<W: AsyncWriteExt + Unpin>(
    state: &Arc<DaemonState>,
    id: u64,
    params: &serde_json::Value,
    writer: &mut W,
) -> Result<()> {
    let p: TailParams = serde_json::from_value(params.clone()).unwrap_or_default();
    let project = p.project.clone();
    let backfill_n = p.backfill.unwrap_or(20);
    let since = p.since.unwrap_or(0);

    // Ensure the requested project is in the union subscription.
    if let Some(pr) = &project {
        let _ = ensure_subscription(state, pr).await;
    }

    // Subscribe BEFORE backfill so we don't miss events that arrive during query.
    let mut rx = state.tail_subscribe();

    {
        *state.open_clients.lock().unwrap() += 1;
        state.liveness_changed.notify_waiters();
    }
    let _guard = ClientGuard(state.clone());

    // ── Backfill ────────────────────────────────────────────────────────────
    if backfill_n > 0 {
        let backfill_events = build_backfill(state, project.as_deref(), backfill_n, since);
        for ev in backfill_events {
            if write_json(writer, &Response::item(id, serde_json::to_value(&ev)?))
                .await
                .is_err()
            {
                let _ = write_json(writer, &Response::end(id)).await;
                return Ok(());
            }
        }
    }

    // ── Live stream ─────────────────────────────────────────────────────────
    loop {
        match rx.recv().await {
            Ok(ev) => {
                if tail_event_matches_project(&ev, project.as_deref())
                    && ev.ts() >= since
                    && write_json(writer, &Response::item(id, serde_json::to_value(&ev)?))
                        .await
                        .is_err()
                {
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
        }
    }
    let _ = write_json(writer, &Response::end(id)).await;
    Ok(())
}

/// True when the event belongs to the requested project scope (or no filter).
pub(in crate::daemon::server) fn tail_event_matches_project(
    ev: &TailEvent,
    project: Option<&str>,
) -> bool {
    let Some(pr) = project else {
        return true;
    };
    let ev_project = match ev {
        TailEvent::Msg { project, .. } => project.as_str(),
        TailEvent::Sync { project, .. } => project.as_str(),
        TailEvent::Turn { project, .. } => project.as_str(),
        TailEvent::Status { project, .. } => project.as_str(),
        TailEvent::Join { project, .. } => project.as_str(),
        TailEvent::Leave { project, .. } => project.as_str(),
        TailEvent::Sess { project, .. } => project.as_str(),
        TailEvent::Proj { project, .. } => project.as_str(),
        // Profiles are cross-project; always include.
        TailEvent::Profile { .. } => return true,
    };
    ev_project == pr
}

/// Build the backfill event list from the canonical read model.
///
/// Returns recent messages as `Msg` events + a roster snapshot of live sessions
/// as synthetic `Join`/`Turn`/`Status` events, sorted by timestamp ascending.
pub(in crate::daemon::server) fn build_backfill(
    state: &Arc<DaemonState>,
    project: Option<&str>,
    limit: u64,
    since: u64,
) -> Vec<TailEvent> {
    let mut events: Vec<TailEvent> = Vec::new();

    // ── Recent chat lines from chat_messages ───────────────────────────────────
    let raw_msgs: Vec<(u64, String, String, String, Option<String>)> = state.with_store(|s| {
        s.recent_chat_for_backfill(project, since, limit)
            .unwrap_or_default()
    });

    for (ts, body, author_pubkey, proj, author_session) in raw_msgs {
        // Resolve slug from pubkey.
        let from_slug = state
            .with_store(|s| s.resolve_slug_for_pubkey(&author_pubkey))
            .ok()
            .flatten()
            .unwrap_or_else(|| pubkey_short(&author_pubkey));
        events.push(TailEvent::Msg {
            ts,
            project: proj,
            from: from_slug,
            from_session: author_session,
            to: String::new(), // backfill: recipient not stored inline
            to_session: None,
            body: body.chars().take(200).collect(),
        });
    }

    // ── Roster snapshot: live sessions ──────────────────────────────────────
    let now = now_secs();
    let since_peer = now.saturating_sub(PRUNE_PEER_AFTER_SECS);

    // Peer sessions as synthetic Join events, status via the SHARED projection.
    let peers = state.with_store(|s| {
        s.peer_session_snapshots(project, since_peer)
            .unwrap_or_default()
    });
    for snap in peers {
        let d = derive_status(&snap, now);
        events.push(TailEvent::Join {
            ts: snap.last_seen,
            project: snap.project.clone(),
            agent: snap.agent_slug.clone(),
            host: snap.host.clone(),
            session: snap.session_id.as_str().to_owned(),
            rel_cwd: snap.rel_cwd.clone(),
        });
        if !d.title.is_empty() || d.busy {
            events.push(TailEvent::Status {
                ts: snap.last_seen,
                project: snap.project.clone(),
                agent: snap.agent_slug.clone(),
                text: d.title.clone(),
                active: d.busy,
            });
        }
    }

    // Own live sessions as synthetic Sess events, busy via the SHARED projection.
    let mine = state.with_store(|s| s.live_session_snapshots(project, 0).unwrap_or_default());
    for snap in mine {
        let d = derive_status(&snap, now);
        events.push(TailEvent::Sess {
            ts: snap.first_seen,
            project: snap.project.clone(),
            agent: snap.agent_slug.clone(),
            session: snap.session_id.as_str().to_owned(),
            state: "start".into(),
            rel_cwd: snap.rel_cwd.clone(),
        });
        if d.busy {
            events.push(TailEvent::Turn {
                ts: snap.turn_started_at,
                project: snap.project.clone(),
                agent: snap.agent_slug.clone(),
                session: snap.session_id.as_str().to_owned(),
                state: "working".into(),
                elapsed_s: None,
            });
        }
    }

    // Sort ascending by timestamp.
    events.sort_by_key(|e| e.ts());
    events
}
