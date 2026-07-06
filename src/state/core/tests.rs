use super::*;

fn reg(ext: &str, channel: &str, now: u64) -> RegisterSession {
    RegisterSession {
        harness: "codex".into(),
        external_id_kind: "harness_session".into(),
        external_id: ext.into(),
        agent_pubkey: format!("pk-{ext}"),
        agent_slug: "agent".into(),
        channel_h: channel.into(),
        child_pid: None,
        transcript_path: None,
        resume_id: String::new(),
        now,
    }
}

#[test]
fn table_samples_prefer_alive_sessions_over_newer_dead_rows() {
    let s = Store::open_memory().unwrap();
    let alive = s.register_session(&reg("alive", "room", 100)).unwrap();
    let dead = s.register_session(&reg("dead", "room", 200)).unwrap();
    s.mark_dead(&dead).unwrap();

    let rows = s
        .application_table_sample_rows("sessions", &["session_id"], 2)
        .unwrap()
        .unwrap();

    assert_eq!(rows[0]["session_id"], alive);
}

#[test]
fn table_samples_prefer_live_alias_and_joined_channel_rows() {
    let s = Store::open_memory().unwrap();
    let alive = s.register_session(&reg("alive", "room", 100)).unwrap();
    let dead = s.register_session(&reg("dead", "room", 200)).unwrap();
    s.mark_dead(&dead).unwrap();

    let aliases = s
        .application_table_sample_rows("session_aliases", &["external_id"], 2)
        .unwrap()
        .unwrap();
    let joined = s
        .application_table_sample_rows("session_channels", &["session_id"], 2)
        .unwrap()
        .unwrap();

    assert_eq!(aliases[0]["external_id"], "alive");
    assert_eq!(joined[0]["session_id"], alive);
}

#[test]
fn table_samples_prefer_fresh_status_rows() {
    let s = Store::open_memory().unwrap();
    s.upsert_status(&Status {
        pubkey: "pk-old".into(),
        session_id: "old".into(),
        channel_h: "room".into(),
        slug: "agent".into(),
        title: String::new(),
        activity: String::new(),
        busy: false,
        last_seen: 100,
        updated_at: 100,
        expiration: 100,
    })
    .unwrap();
    s.upsert_status(&Status {
        pubkey: "pk-fresh".into(),
        session_id: "fresh".into(),
        channel_h: "room".into(),
        slug: "agent".into(),
        title: String::new(),
        activity: String::new(),
        busy: false,
        last_seen: 200,
        updated_at: 200,
        expiration: 300,
    })
    .unwrap();

    let rows = s
        .application_table_sample_rows("relay_status", &["session_id"], 2)
        .unwrap()
        .unwrap();

    assert_eq!(rows[0]["session_id"], "fresh");
}
