use crate::daemon_harness::*;
use tenex_edge::daemon::client::Client;
use tenex_edge::state::Store;

#[test]
fn cli_my_status_sets_the_exact_pty_session_topic() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new().with_backend_key();
    let session_id = rt().block_on(async {
        let mut client = Client::connect_or_spawn().await.expect("connect");
        let response = client
            .call(
                "session_start",
                serde_json::json!({
                    "agent": "codex",
                    "harness": "codex",
                    "session_id": "native-topic-session",
                    "cwd": "/tmp",
                    "pty_session": "pty-topic-session",
                }),
            )
            .await
            .expect("session start");
        response["session_id"]
            .as_str()
            .expect("canonical session id")
            .to_string()
    });

    let topic = "Researching MCP improvements around resource allocation";
    let out = run_cli_with_env(
        &home,
        &["my", "status", "--topic", topic],
        &[("TENEX_EDGE_PTY_SESSION", "pty-topic-session")],
    );
    assert!(
        out.status.success(),
        "my status failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout)
        .contains("automatic distillation paused for 30 minutes"));

    let rec = Store::open(&home.store_path())
        .unwrap()
        .get_session(&session_id)
        .unwrap()
        .expect("session row");
    assert_eq!(rec.work_topic, topic);
    assert!(rec.work_topic_set_at > 0);

    stop_daemon(&home);
}
