use super::*;

pub(in crate::daemon::server) struct ChatRecordDraft {
    from_pubkey: String,
    from_slug: String,
    host: String,
    project: String,
    body: String,
    from_session: String,
    mentioned_session: String,
}

pub(in crate::daemon::server) async fn publish_chat_checked(
    state: &Arc<DaemonState>,
    chat: &ChatMessage,
    signing: &Keys,
    draft: &ChatRecordDraft,
) -> Result<String> {
    let builder = state
        .provider
        .encode(&DomainEvent::ChatMessage(chat.clone()))?;
    let signed = state.transport.sign(builder, signing).await?;
    let event_id = signed.id.to_hex();
    let created_at = now_secs();

    state.with_store(|s| {
        let _ = s.record_chat(&ChatLogRow {
            chat_event_id: event_id.clone(),
            from_pubkey: draft.from_pubkey.clone(),
            from_slug: draft.from_slug.clone(),
            host: draft.host.clone(),
            project: draft.project.clone(),
            body: draft.body.clone(),
            created_at,
            from_session: draft.from_session.clone(),
            mentioned_session: draft.mentioned_session.clone(),
        });
    });

    state.transport.publish_event(&signed).await?;
    Ok(event_id)
}

pub(in crate::daemon::server) fn spawn_chat_publish_retry(
    state: Arc<DaemonState>,
    chat: ChatMessage,
    signing: Keys,
    draft: ChatRecordDraft,
    label: &'static str,
) {
    tokio::spawn(async move {
        let mut last_err = String::new();
        for attempt in 0..60 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
            match publish_chat_checked(&state, &chat, &signing, &draft).await {
                Ok(_) => return,
                Err(e) => last_err = e.to_string(),
            }
        }
        eprintln!("[daemon] {label} kind:9 publish retry exhausted: {last_err}");
    });
}

pub(in crate::daemon::server) fn spawn_retry_drainer(state: Arc<DaemonState>) {
    let queue = state.transport.retry_queue.clone();
    let transport = state.transport.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let due = queue.drain_due();
            for retry in due {
                let id_short = {
                    let h = retry.event.id.to_hex();
                    h[..8.min(h.len())].to_string()
                };
                let kind = retry.event.kind.as_u16();
                if transport.retry_publish(&retry.event).await {
                    eprintln!(
                        "[retry] event {id_short} kind:{kind} accepted on attempt {}",
                        retry.attempt + 1
                    );
                } else {
                    queue.requeue(retry, "relay rejected on retry");
                }
            }
        }
    });
}

pub(in crate::daemon::server) async fn apply_room_name_update(
    state: &Arc<DaemonState>,
    room: &str,
    title: &str,
) -> bool {
    let title = title.trim();
    if title.is_empty() {
        return false;
    }
    let should_publish = state.with_store(|s| {
        let owned =
            s.is_session_room(room).unwrap_or(false) && s.is_group_owned(room).unwrap_or(false);
        let current = s.group_display_name(room).unwrap_or_default();
        owned && current.trim() != title
    });
    if !should_publish {
        return false;
    }
    let renamed = state.provider.nip29_set_group_name(room, title).await;
    if renamed {
        let parent = state
            .with_store(|s| s.session_room_parent(room).ok().flatten())
            .unwrap_or_default();
        state.with_store(|s| {
            s.upsert_group_metadata(room, title, &parent, now_secs())
                .ok();
        });
    }
    renamed
}

pub(in crate::daemon::server) fn spawn_room_name_update(
    state: Arc<DaemonState>,
    room: String,
    title: String,
) {
    tokio::spawn(async move {
        apply_room_name_update(&state, &room, &title).await;
    });
}

/// Publish the agent's completed-turn output as kind:9 chat into the session's
/// room (issue #6). Signed by the agent via `keys_for_session`, which falls
/// back to the durable agent key (#5). Mirrors `rpc_chat_write`'s publish +
/// local record, minus mention handling.
pub(in crate::daemon::server) async fn publish_agent_reply(
    state: &Arc<DaemonState>,
    rec: &crate::state::SessionRecord,
    reply: &str,
) -> Result<()> {
    let id = identity::load_or_create(&config::edge_home(), &rec.agent_slug, now_secs())?;
    // Sign with the selected session identity: durable by default, transient
    // only when this live session collided with another durable signer in the
    // same routing scope.
    let signing = state
        .keys_for_session(&rec.session_id)
        .unwrap_or_else(|| id.keys.clone());
    let from_pubkey = signing.public_key().to_hex();
    // Route into the session's CURRENT scope (channel when set, else the
    // per-session room) so a `channels switch` moves the agent's turn replies
    // to the new room without restarting.
    let scope = rec.route_scope().to_string();

    let chat = ChatMessage {
        from: crate::domain::AgentRef::new(from_pubkey.clone(), rec.agent_slug.clone()),
        project: scope.clone(),
        body: reply.to_string(),
        mentioned_pubkey: None,
    };
    let draft = ChatRecordDraft {
        from_pubkey,
        from_slug: rec.agent_slug.clone(),
        host: state.host.clone(),
        project: scope,
        body: reply.to_string(),
        from_session: rec.session_id.clone(),
        mentioned_session: String::new(),
    };
    let publish = publish_chat_checked(state, &chat, &signing, &draft);
    match tokio::time::timeout(std::time::Duration::from_secs(3), publish).await {
        Ok(Ok(_)) => {}
        Ok(Err(_)) | Err(_) => {
            spawn_chat_publish_retry(state.clone(), chat, signing, draft, "agent reply");
        }
    }
    Ok(())
}

/// Publish a user's prompt as kind:9 chat into the session's room (issue #6).
///
/// The human is speaking, so the event is signed by the OPERATOR key (which is
/// the room's admin) rather than the agent/session key. Fail-open: if no
/// operator key is set the prompt is simply not mirrored — the hook must never
/// block the editor. The session resolves to its room via `resolve_session`,
/// so the prompt lands in the same per-session subgroup the agent posts into.
pub(in crate::daemon::server) async fn rpc_user_prompt(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    use nostr_sdk::prelude::Keys;

    #[derive(serde::Deserialize, Default)]
    struct P {
        #[serde(default)]
        session: Option<String>,
        #[serde(default)]
        env_session: Option<String>,
        #[serde(default)]
        agent: Option<String>,
        #[serde(default)]
        cwd: Option<String>,
        #[serde(default)]
        prompt: String,
    }
    let p: P = serde_json::from_value(params.clone()).context("parsing user_prompt params")?;
    if p.prompt.trim().is_empty() {
        return Ok(serde_json::json!({ "skipped": "empty prompt" }));
    }

    // A daemon-injected fabric envelope (a mention pasted into the pane by the
    // tmux delivery path) is ALREADY a kind:9 event in the room. The harness
    // re-submits it as a prompt, firing user-prompt-submit, but republishing it
    // would echo the message back into the channel (and, on publish timeout +
    // retry, twice). Only genuine human keyboard input is mirrored.
    if crate::injection::is_fabric_injection(&p.prompt) {
        return Ok(serde_json::json!({ "skipped": "fabric injection echo" }));
    }

    // No operator key → nothing to sign with; fail open (session still runs).
    // `userNsec` is the ONLY signer for user prompts — the human is speaking.
    let Some(nsec) = state.cfg.user_nsec() else {
        return Ok(serde_json::json!({ "skipped": "userNsec unset" }));
    };
    let op_keys = Keys::parse(nsec).context("parsing operator key")?;
    let op_pubkey = op_keys.public_key().to_hex();

    let rec = resolve_session(
        state,
        p.session.as_deref(),
        p.env_session.as_deref(),
        p.cwd.as_deref(),
        p.agent.as_deref(),
        None,
    )?;

    // Only mirror prompts into a per-session room. A human start with no resume
    // anchor (or no operator key) keeps `project == work_root`; mirroring there
    // would spam the bare project group, so skip it (fail-open). Gate on the
    // local `is_session_room` flag — set synchronously at mint and never touched
    // by the relay materializer (unlike `project_meta.parent`, which a relay that
    // doesn't re-emit the NIP-29 parent tag can clobber to empty).
    let is_room = state
        .with_store(|s| s.is_session_room(&rec.project))
        .unwrap_or(false);
    if !is_room {
        return Ok(serde_json::json!({ "skipped": "session not in a room" }));
    }

    // Seed the canonical session title once, from the real user prompt, so the
    // status pipeline has an immediate title before the runtime transcript read
    // catches up. Room metadata is a consequence of a title transition, not a
    // per-hook mirror: publish a room-name update only when this call actually
    // changed session_state.
    let seeded_title = state
        .with_store(|s| s.local_session_snapshot(&rec.session_id).ok().flatten())
        .and_then(|snap| {
            if snap.title_source == crate::session::TitleSource::None {
                let seed = crate::util::titleize_prompt(&p.prompt);
                if seed.is_empty() {
                    return None;
                }
                state
                    .with_store(|s| {
                        s.seed_title_if_empty(&rec.session_id, snap.turn_id, &seed, now_secs())
                            .ok()
                            .flatten()
                    })
                    .and_then(|updated| {
                        let title = updated.title.trim();
                        (!title.is_empty()).then(|| title.to_string())
                    })
            } else {
                None
            }
        });
    if let Some(room_title) = seeded_title {
        state.status_outbox_notify.notify_waiters();
        spawn_room_name_update(state.clone(), rec.project.clone(), room_title);
    }

    // Publish into the session's CURRENT routing scope — its channel when set
    // (a `channels switch` moved it to a subgroup), else its per-session room.
    // The `is_room` gate above still keys on `project` (the minted room flag is
    // stable across a switch), but the wire `h` tag must follow the channel so
    // the prompt lands where the rest of the session's chat now goes.
    let scope = rec.route_scope().to_string();
    let chat = ChatMessage {
        from: crate::domain::AgentRef::new(op_pubkey.clone(), "operator".to_string()),
        project: scope.clone(),
        body: p.prompt.clone(),
        mentioned_pubkey: None,
    };
    let draft = ChatRecordDraft {
        from_pubkey: op_pubkey,
        from_slug: "operator".to_string(),
        host: state.host.clone(),
        project: scope.clone(),
        body: p.prompt.clone(),
        from_session: rec.session_id.clone(),
        mentioned_session: String::new(),
    };
    // Try once synchronously so local chat history exists when the hook/RPC
    // returns. If relay membership is still converging, fall back to a daemon
    // retry without blocking the editor.
    let publish = publish_chat_checked(state, &chat, &op_keys, &draft);
    match tokio::time::timeout(std::time::Duration::from_secs(3), publish).await {
        Ok(Ok(_)) => {}
        Ok(Err(_)) | Err(_) => {
            spawn_chat_publish_retry(state.clone(), chat, op_keys, draft, "user prompt");
        }
    }

    Ok(serde_json::json!({ "queued": true, "project": scope }))
}
