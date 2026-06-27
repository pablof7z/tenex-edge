use super::{
    materialize_member_snapshot, refresh_project_members, rewrite_config_with_user_nsec,
    unique_session, write_config,
};
use crate::daemon_harness::{rt, shared_nip29_relay_url, stop_daemon, wait_until, Home, ENV_LOCK};
use nostr_sdk::prelude::{Client as NostrClient, ClientOptions, EventBuilder, Filter, Keys, Kind};
use nostr_sdk::NostrSigner;
use tenex_edge::daemon::client::Client;
use tenex_edge::fabric::nip29::wire::KIND_PROFILE;
use tenex_edge::state::Store;

async fn publish_profile(keys: &Keys, name: &str) {
    let client = NostrClient::builder()
        .signer(keys.clone())
        .opts(ClientOptions::default().automatic_authentication(true))
        .build();
    client
        .add_relay(shared_nip29_relay_url())
        .await
        .expect("add relay");
    client.connect().await;
    client
        .wait_for_connection(std::time::Duration::from_secs(8))
        .await;
    let _ = client
        .fetch_events(
            Filter::new().kind(Kind::from(0u16)).limit(1),
            std::time::Duration::from_secs(5),
        )
        .await;
    let builder = EventBuilder::new(
        Kind::from(KIND_PROFILE),
        serde_json::json!({ "name": name }).to_string(),
    );
    let unsigned = builder.build(keys.public_key());
    let signed = keys.sign_event(unsigned).await.expect("sign profile");
    let out = client.send_event(&signed).await.expect("publish profile");
    assert!(
        !out.success.is_empty(),
        "profile publish rejected: success={:?} failed={:?}",
        out.success,
        out.failed
    );
}

/// e2e: a human-initiated session's first turn gets the channel-hierarchy
/// context block, rendered through the real daemon.
#[test]
fn first_turn_injects_channel_context_block() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_user_nsec(&home);

    let ctx = rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": "sess-ctx-1", "cwd": "/tmp", "watch_pid": std::process::id()}),
        )
        .await
        .expect("session_start");
        let v = c
            .call("turn_start", serde_json::json!({"session": "sess-ctx-1"}))
            .await
            .expect("turn_start");
        v.get("context")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string()
    });

    assert!(
        ctx.contains("[tenex-edge] Fabric context"),
        "context was: {ctx}"
    );
    assert!(
        !ctx.contains("[session"),
        "must not expose a session code; context was: {ctx}"
    );
    assert!(ctx.contains("Channel: #"), "context was: {ctx}");
    assert!(ctx.contains("@coder (you)"), "context was: {ctx}");

    stop_daemon(&home);
}

#[test]
fn first_turn_resolves_member_profiles_from_kind0() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    write_config(&home, false);
    let sid = unique_session("sess-member-profile");
    let remote = Keys::generate();
    let remote_pk = remote.public_key().to_hex();
    let remote_name = "profiled-member";

    let ctx = rt().block_on(async {
        publish_profile(&remote, remote_name).await;
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": sid, "cwd": "/tmp", "watch_pid": std::process::id()}),
        )
        .await
        .expect("session_start");
        c.call(
            "project_add",
            serde_json::json!({"project": "tmp", "pubkey": remote_pk}),
        )
        .await
        .expect("project_add profiled member");
        let members = c
            .call("project_members", serde_json::json!({"project": "tmp"}))
            .await
            .expect("project_members");
        assert!(
            members["members"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["pubkey"] == remote_pk && m["slug"] == remote_name),
            "project_members should resolve kind:0 slugs: {members}"
        );
        let v = c
            .call("turn_start", serde_json::json!({"session": sid}))
            .await
            .expect("turn_start");
        v.get("context")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string()
    });

    assert!(
        ctx.contains(&format!("@{remote_name} - offline")),
        "member roster should resolve kind:0 profile; context was: {ctx}"
    );
    assert!(
        !ctx.contains(&format!("@{}", &remote_pk[..8])),
        "member roster should not fall back to raw pubkey short; context was: {ctx}"
    );

    stop_daemon(&home);
}

#[test]
fn session_start_with_user_nsec_owns_group_and_adds_member() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_user_nsec(&home);

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": "sess-grp-1", "cwd": "/tmp", "watch_pid": std::process::id()}),
        )
        .await
        .expect("session_start");
    });

    let store = Store::open(&home.store_path()).unwrap();
    let rec = store
        .get_session("sess-grp-1")
        .unwrap()
        .expect("session row");
    assert!(rec.alive);
    assert_eq!(
        store.session_room_parent(&rec.project).unwrap().as_deref(),
        Some("tmp"),
        "session start should record the local-only room parent"
    );

    stop_daemon(&home);
}

/// Human-initiated sessions with per-session rooms enabled mint child rooms
/// under the work-root project.
#[test]
fn human_initiated_session_mints_per_session_room() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_user_nsec(&home);
    let sid = unique_session("sess-room");

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": sid, "cwd": "/tmp"}),
        )
        .await
        .expect("session_start");
    });

    let store = Store::open(&home.store_path()).unwrap();
    let rec = store.get_session(&sid).unwrap().expect("session row");
    assert_ne!(
        rec.project, "tmp",
        "human-initiated session should mint a per-session room, not use the bare project"
    );
    assert!(
        rec.project.starts_with("session-"),
        "room id should be project-agnostic: got {}",
        rec.project
    );
    let breadcrumb = store.channel_breadcrumb(&rec.project).unwrap();
    assert_eq!(
        breadcrumb.last().map(|(_, label)| label.as_str()),
        Some(rec.project.as_str()),
        "session room display label should be the room id"
    );

    materialize_member_snapshot(&home, &rec.project, &rec.agent_pubkey);
    assert!(
        wait_until(std::time::Duration::from_secs(20), || Store::open(
            &home.store_path()
        )
        .map(|s| {
            refresh_project_members(&rec.project);
            s.is_group_member(&rec.project, &rec.agent_pubkey)
                .unwrap_or(false)
        })
        .unwrap_or(false)),
        "the agent should be a member of its per-session room"
    );

    stop_daemon(&home);
}

/// With per-session rooms disabled, a human-initiated session uses the work-root
/// project channel.
#[test]
fn human_initiated_session_uses_project_when_per_session_rooms_disabled() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    write_config(&home, false);
    let sid = unique_session("sess-noroom");

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": sid, "cwd": "/tmp"}),
        )
        .await
        .expect("session_start");
    });

    let store = Store::open(&home.store_path()).unwrap();
    let rec = store.get_session(&sid).unwrap().expect("session row");
    assert_eq!(
        rec.project, "tmp",
        "with per-session rooms disabled, the session should use the project channel"
    );
    assert!(
        !rec.project.starts_with("session-"),
        "no per-session room should be minted: got {}",
        rec.project
    );
    assert!(
        !store.is_session_room(&rec.project).unwrap(),
        "the work-root project is not a session room"
    );

    stop_daemon(&home);
}

/// Opencode-style human sessions have no harness/resume id, so the room anchor
/// falls back to the watched pid.
#[test]
fn opencode_style_session_without_id_mints_room_via_pid() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_user_nsec(&home);

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "opencoder", "cwd": "/tmp", "watch_pid": std::process::id()}),
        )
        .await
        .expect("session_start");
    });

    let store = Store::open(&home.store_path()).unwrap();
    let rec = store
        .list_alive_sessions()
        .unwrap()
        .into_iter()
        .find(|r| r.agent_slug == "opencoder")
        .expect("opencode session row");
    assert!(
        rec.project.starts_with("session-"),
        "opencode session must mint a per-session room: got {}",
        rec.project
    );
    assert!(
        store.is_session_room(&rec.project).unwrap(),
        "minted group must be flagged as a per-session room"
    );

    stop_daemon(&home);
}
