use super::*;

/// A user's prompt is published as kind:9 chat into the session's room
/// (operator-signed -- the human is speaking, and the operator is the room
/// admin). (Issue #6, increment 3.)
#[test]
fn user_prompt_publishes_kind9_chat_into_room() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_user_nsec(&home);
    let sid = unique_session("sess-prompt");

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": sid, "cwd": "/tmp", "watch_pid": std::process::id()}),
        )
        .await
        .expect("session_start");
    });

    // The room is minted on the relay in the background; wait until the agent is
    // a member (room fully live) before mirroring a prompt into it.
    let rec = Store::open(&home.store_path())
        .unwrap()
        .get_session(&sid)
        .unwrap()
        .expect("session row");
    materialize_member_snapshot(&home, &rec.channel_h, &rec.agent_pubkey);
    assert!(
        wait_until(std::time::Duration::from_secs(20), || Store::open(
            &home.store_path()
        )
        .map(|s| {
            refresh_project_members(&rec.channel_h);
            s.is_channel_member(&rec.channel_h, &rec.agent_pubkey)
                .unwrap_or(false)
        })
        .unwrap_or(false)),
        "room {} not live in time",
        rec.channel_h
    );

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "user_prompt",
            serde_json::json!({"env_session": sid, "agent": "coder", "cwd": "/tmp", "prompt": "build me a thing"}),
        )
        .await
        .expect("user_prompt");
    });

    let store = Store::open(&home.store_path()).unwrap();
    let session = store.get_session(&sid).unwrap().expect("session row");
    assert_eq!(
        session.title, "",
        "user_prompt must not seed the kind:30315 title from the prompt"
    );
    let msgs = chat_in_channel(&store, &rec.channel_h);
    assert!(
        msgs.iter().any(|m| m.content == "build me a thing"),
        "user prompt should be recorded as chat in room {}; got {:?}",
        rec.channel_h,
        msgs.iter().map(|m| &m.content).collect::<Vec<_>>()
    );

    stop_daemon(&home);
}

/// A prompt Codex dispatches to a subagent it spawned (spawn_agent/
/// multi_agent_v1*) re-fires the same hook on the same session_id as a real
/// human prompt, carrying a `subagent_id`. It must NOT be mirrored as chat --
/// the human didn't say it. (Issue #102.)
#[test]
fn subagent_dispatch_prompt_is_not_mirrored_as_chat() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_user_nsec(&home);
    let sid = unique_session("sess-subagent-prompt");

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": sid, "cwd": "/tmp", "watch_pid": std::process::id()}),
        )
        .await
        .expect("session_start");
    });

    let rec = Store::open(&home.store_path())
        .unwrap()
        .get_session(&sid)
        .unwrap()
        .expect("session row");
    materialize_member_snapshot(&home, &rec.channel_h, &rec.agent_pubkey);
    assert!(
        wait_until(std::time::Duration::from_secs(20), || Store::open(
            &home.store_path()
        )
        .map(|s| {
            refresh_project_members(&rec.channel_h);
            s.is_channel_member(&rec.channel_h, &rec.agent_pubkey)
                .unwrap_or(false)
        })
        .unwrap_or(false)),
        "agent should become a room member"
    );

    let resp = rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "user_prompt",
            serde_json::json!({
                "env_session": sid,
                "agent": "coder",
                "cwd": "/tmp",
                "prompt": "Please return exactly one random color name as your response.",
                "subagent_id": "019f1cad-51b7-7b32-a340-65e53e932f43",
            }),
        )
        .await
        .expect("user_prompt")
    });
    assert_eq!(
        resp.get("skipped").and_then(|v| v.as_str()),
        Some("subagent dispatch, not human input")
    );

    let store = Store::open(&home.store_path()).unwrap();
    let msgs = chat_in_channel(&store, &rec.channel_h);
    assert!(
        !msgs.iter().any(|m| m.content.contains("random color name")),
        "subagent dispatch prompt must not be mirrored as chat; got {:?}",
        msgs.iter().map(|m| &m.content).collect::<Vec<_>>()
    );

    stop_daemon(&home);
}

/// When the agent finishes a turn (stop hook), its turn output is published as
/// kind:9 chat into the session's room, signed by the agent's DURABLE identity
/// (via keys_for_session -> durable fallback). (Issue #6, increment 4.)
#[test]
fn agent_reply_publishes_kind9_chat_into_room() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_user_nsec(&home);
    let sid = unique_session("sess-reply");

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": sid, "cwd": "/tmp", "watch_pid": std::process::id()}),
        )
        .await
        .expect("session_start");
    });

    // Wait for the background mint to make the room live before driving a turn.
    let rec = Store::open(&home.store_path())
        .unwrap()
        .get_session(&sid)
        .unwrap()
        .expect("session row");
    materialize_member_snapshot(&home, &rec.channel_h, &rec.agent_pubkey);
    assert!(
        wait_until(std::time::Duration::from_secs(20), || Store::open(
            &home.store_path()
        )
        .map(|s| {
            refresh_project_members(&rec.channel_h);
            s.is_channel_member(&rec.channel_h, &rec.agent_pubkey)
                .unwrap_or(false)
        })
        .unwrap_or(false)),
        "room {} not live in time",
        rec.channel_h
    );

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        // Open a turn so the stop-hook reply publish (gated on was_working) fires.
        c.call("turn_start", serde_json::json!({"session": sid}))
            .await
            .expect("turn_start");
        c.call(
            "turn_end",
            serde_json::json!({"session": sid, "reply": "I fixed the bug in auth.rs"}),
        )
        .await
        .expect("turn_end");
    });

    let store = Store::open(&home.store_path()).unwrap();
    let msgs = chat_in_channel(&store, &rec.channel_h);
    let reply = msgs
        .iter()
        .find(|m| m.content == "I fixed the bug in auth.rs");
    assert!(
        reply.is_some(),
        "agent reply should be chat in room {}; got {:?}",
        rec.channel_h,
        msgs.iter().map(|m| &m.content).collect::<Vec<_>>()
    );
    // The reply is signed by the durable agent identity (the room member), so
    // chat and presence stay on one identity.
    assert_eq!(
        reply.unwrap().pubkey,
        rec.agent_pubkey,
        "agent reply must be signed by the durable agent identity"
    );

    stop_daemon(&home);
}
