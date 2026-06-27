use super::unique_session;
use crate::daemon_harness::{rt, stop_daemon, Home, ENV_LOCK};
use tenex_edge::daemon::client::Client;
use tenex_edge::state::Store;

#[test]
fn agent_reply_publishes_kind9_chat_into_explicit_channel() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    let sid = unique_session("sess-reply-channel");
    let channel = unique_session("reply-channel");
    let reply = "I finished the explicit channel investigation";

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({
                "agent": "coder",
                "session_id": sid,
                "cwd": "/tmp",
                "channel": channel,
                "watch_pid": std::process::id(),
            }),
        )
        .await
        .expect("session_start");
    });

    let rec = Store::open(&home.store_path())
        .unwrap()
        .get_session(&sid)
        .unwrap()
        .expect("session row");
    assert_eq!(rec.route_scope(), channel);
    assert_eq!(rec.project, channel);

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call("turn_start", serde_json::json!({"session": sid}))
            .await
            .expect("turn_start");
        c.call(
            "turn_end",
            serde_json::json!({"session": sid, "reply": reply}),
        )
        .await
        .expect("turn_end");
    });

    let store = Store::open(&home.store_path()).unwrap();
    let msgs = store
        .list_chat_messages(&channel, 0, None, 0, false)
        .unwrap();
    let published = msgs.iter().find(|m| m.body == reply);
    assert!(
        published.is_some(),
        "agent reply should be chat in channel {channel}; got {:?}",
        msgs.iter().map(|m| &m.body).collect::<Vec<_>>()
    );
    assert_eq!(published.unwrap().from_pubkey, rec.agent_pubkey);

    stop_daemon(&home);
}
