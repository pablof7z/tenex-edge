use super::{table_exists, Connection, Store};

#[test]
fn deployed_schema_five_is_rebuilt_into_the_canonical_schema() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE sessions (
            session_id TEXT PRIMARY KEY, agent_pubkey TEXT NOT NULL,
            agent_slug TEXT NOT NULL, channel_h TEXT NOT NULL, harness TEXT NOT NULL,
            child_pid INTEGER, transcript_path TEXT, alive INTEGER NOT NULL,
            created_at INTEGER NOT NULL, last_seen INTEGER NOT NULL, working INTEGER NOT NULL,
            turn_started_at INTEGER NOT NULL, last_distill_at INTEGER NOT NULL,
            work_topic TEXT NOT NULL, work_topic_set_at INTEGER NOT NULL,
            seen_cursor INTEGER NOT NULL, title TEXT NOT NULL, activity TEXT NOT NULL,
            resume_id TEXT NOT NULL, distill_fail_streak INTEGER NOT NULL,
            distill_notice_at INTEGER NOT NULL, explicit_chat_published_at INTEGER NOT NULL
        );
        CREATE INDEX idx_sessions_alive ON sessions(alive, channel_h);
        CREATE TABLE session_channels (
            session_id TEXT NOT NULL, channel_h TEXT NOT NULL, joined_at INTEGER NOT NULL,
            PRIMARY KEY (session_id, channel_h)
        );
        CREATE INDEX idx_session_channels_channel ON session_channels(channel_h, session_id);
        CREATE TABLE session_aliases (
            harness TEXT NOT NULL, external_id_kind TEXT NOT NULL,
            external_id TEXT NOT NULL, session_id TEXT NOT NULL, created_at INTEGER NOT NULL,
            PRIMARY KEY (harness, external_id_kind, external_id)
        );
        CREATE TABLE session_claims (
            pubkey TEXT NOT NULL, agent_slug TEXT NOT NULL, codename TEXT NOT NULL,
            session_id TEXT NOT NULL, channel_h TEXT NOT NULL, native_id TEXT NOT NULL,
            harness TEXT NOT NULL, last_active_at INTEGER NOT NULL, expires_at INTEGER NOT NULL,
            owner_backend_pubkey TEXT NOT NULL, owner_host TEXT NOT NULL,
            PRIMARY KEY (pubkey, channel_h)
        );
        CREATE INDEX idx_session_claims_expires ON session_claims(expires_at);
        CREATE TABLE llm_calls (
            id INTEGER PRIMARY KEY, session_id TEXT NOT NULL, window_hash TEXT NOT NULL,
            provider TEXT NOT NULL, model TEXT NOT NULL, system_prompt TEXT NOT NULL,
            transcript_slice TEXT NOT NULL, raw_response TEXT NOT NULL,
            parsed_title TEXT, parsed_activity TEXT, created_at INTEGER NOT NULL
        );
        CREATE INDEX idx_llm_calls_pubkey ON llm_calls(session_id, created_at);
        CREATE INDEX idx_llm_calls_window_hash ON llm_calls(window_hash);
        "#,
    )
    .unwrap();
    conn.execute(
        "INSERT INTO sessions VALUES (?1, ?2, 'codex', '/mosaico', 'codex', 42, \
         '/tmp/transcript', 1, 1, 2, 1, 3, 4, 'schema upgrade', 5, 6, 'working', \
         'migrating', 'native-1', 7, 8, 9)",
        ["session-1", "pubkey-1"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO session_channels VALUES ('session-1', '/mosaico', 10)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO session_aliases VALUES ('codex', 'harness_session', 'native-1', 'session-1', 11)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO session_claims VALUES ('pubkey-1', 'codex', 'old-name', 'session-1', \
         '/mosaico', 'native-1', 'codex', 12, 13, 'backend-1', 'host-1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO llm_calls VALUES (1, 'session-1', 'window-1', 'provider', 'model', \
         'system', 'transcript', 'raw', 'title', 'activity', 14)",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 5u32).unwrap();
    drop(conn);

    drop(Store::open(&path).expect("deployed schema five migrates"));

    let conn = Connection::open(&path).unwrap();
    assert_eq!(
        conn.pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
            .unwrap(),
        6
    );
    assert!(!table_exists(&conn, "session_aliases"));
    assert_eq!(
        conn.query_row(
            "SELECT pubkey, runtime_generation, title FROM sessions",
            [],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u32>(1)?,
                row.get::<_, String>(2)?
            )),
        )
        .unwrap(),
        ("pubkey-1".into(), 0, "working".into())
    );
    assert_eq!(
        conn.query_row(
            "SELECT locator_kind, locator_value, pubkey FROM session_locators",
            [],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?
            )),
        )
        .unwrap(),
        ("native_resume".into(), "native-1".into(), "pubkey-1".into())
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM llm_calls WHERE pubkey='pubkey-1'",
            [],
            |row| { row.get::<_, u32>(0) }
        )
        .unwrap(),
        1
    );
}
