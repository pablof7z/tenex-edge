use super::*;

/// An orchestration-spawned session (the backend set `TENEX_EDGE_CHANNEL` to add
/// this agent to a task subgroup) joins that group as-is and does NOT mint a
/// child room. Guards the discriminator boundary.
#[test]
fn orchestration_session_uses_existing_group_without_minting() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_user_nsec(&home);

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": "sess-orch-1", "cwd": "/tmp", "channel": "issue-42"}),
        )
        .await
        .expect("session_start");
    });

    let store = Store::open(&home.store_path()).unwrap();
    let rec = store
        .get_session("sess-orch-1")
        .unwrap()
        .expect("session row");
    assert_eq!(
        rec.channel_h, "issue-42",
        "with a channel override the session joins it; it must not mint a child room"
    );

    stop_daemon(&home);
}
