use super::*;

fn sample_session(id: &str) -> SessionRecord {
    SessionRecord {
        session_id: id.into(),
        agent_slug: "coder".into(),
        agent_pubkey: "pk-coder".into(),
        project: "proj".into(),
        host: "laptop".into(),
        child_pid: Some(42),
        watch_pid: Some(7),
        created_at: 1000,
        alive: true,
        rel_cwd: String::new(),
        channel: String::new(),
    }
}

// ── Phase 1: canonical read-model schema ─────────────────────────────

#[test]
fn phase1_new_tables_exist_after_open() {
    let s = Store::open_memory().unwrap();
    let n: i64 = s
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN
             ('projects','project_origins','inbound_quarantine','membership')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 4, "all 4 Phase 1 tables must be created");
}

// ── Phase 2: read-model and write-facing materializer unit tests ─────────

/// list_projects_read_model delegates to list_project_meta — same rows, same order.
#[test]
fn phase2_list_projects_read_model_matches_project_meta() {
    let s = Store::open_memory().unwrap();
    assert!(s.list_projects_read_model().unwrap().is_empty());
    s.upsert_project_meta("zap", "about-zap", 1).unwrap();
    s.upsert_project_meta("alpha", "about-alpha", 2).unwrap();
    let rows = s.list_projects_read_model().unwrap();
    // list_project_meta orders by project slug.
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, "alpha");
    assert_eq!(rows[1].0, "zap");
    assert_eq!(rows[0].1, "about-alpha");
}

/// project_meta_read_model is a pass-through of get_project_meta.
#[test]
fn phase2_project_meta_read_model_passthrough() {
    let s = Store::open_memory().unwrap();
    assert!(s.project_meta_read_model("missing").unwrap().is_none());
    s.upsert_project_meta("proj", "the about", 1).unwrap();
    assert_eq!(
        s.project_meta_read_model("proj").unwrap().as_deref(),
        Some("the about")
    );
}

/// list_agents_read_model returns alive sessions filtered by project + freshness.
#[test]
fn phase2_list_agents_read_model_filters() {
    let s = Store::open_memory().unwrap();
    let mut r = sample_session("s1");
    r.project = "proj".into();
    s.upsert_session(&r).unwrap();
    s.touch_session("s1", 1000).unwrap();

    let mut r2 = sample_session("s2");
    r2.project = "other".into();
    s.upsert_session(&r2).unwrap();
    s.touch_session("s2", 1000).unwrap();

    // Project-scoped.
    let proj = s.list_agents_read_model(Some("proj"), 0).unwrap();
    assert_eq!(proj.len(), 1);
    assert_eq!(proj[0].session_id, "s1");

    // Freshness filter: since=1001 → both stale.
    assert!(s.list_agents_read_model(None, 1001).unwrap().is_empty());

    // All projects, no freshness filter.
    assert_eq!(s.list_agents_read_model(None, 0).unwrap().len(), 2);
}

/// list_presence_read_model delegates to list_peer_sessions.
#[test]
fn phase2_list_presence_read_model_delegates() {
    let s = Store::open_memory().unwrap();
    s.upsert_peer_session("ps1", "pk-a", "agentA", "proj", "host", "", 500)
        .unwrap();
    let rows = s.list_presence_read_model(Some("proj"), 0).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].slug, "agentA");
    // Since filter.
    assert!(s
        .list_presence_read_model(Some("proj"), 600)
        .unwrap()
        .is_empty());
}

/// materialize_profile round-trips through upsert_profile.
#[test]
fn phase2_materialize_profile() {
    let s = Store::open_memory().unwrap();
    s.materialize_profile("pk-mp", "agent-mp", "host-mp", 100)
        .unwrap();
    let pk = s.resolve_agent_pubkey("agent-mp", None).unwrap();
    assert_eq!(pk.as_deref(), Some("pk-mp"));
}

/// materialize_presence round-trips through upsert_peer_session.
#[test]
fn phase2_materialize_presence() {
    let s = Store::open_memory().unwrap();
    s.materialize_presence(
        "sess-mp", "pk-mp", "agent-mp", "proj", "host", "subdir", 100,
    )
    .unwrap();
    let rows = s.list_presence_read_model(Some("proj"), 0).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].rel_cwd, "subdir");
}

/// record_peer_status mirrors a kind:30315 into peer_session_state and bumps
/// state_version only on content change.
#[test]
fn record_peer_status_upserts_and_versions() {
    use crate::session::PeerStatusObservation;
    let s = Store::open_memory().unwrap();
    let mut obs = PeerStatusObservation {
        agent_pubkey: "pk-peer".into(),
        agent_slug: "peer".into(),
        project: "proj".into(),
        host: "host2".into(),
        rel_cwd: String::new(),
        title: "fixing auth".into(),
        activity: "editing".into(),
        busy: true,
        emitted_at: 100,
        observed_at: 100,
    };
    s.record_peer_status(&obs).unwrap();
    let snaps = s.peer_session_snapshots(Some("proj"), 0).unwrap();
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].title, "fixing auth");
    assert_eq!(snaps[0].state_version, 1);
    // Same content, newer emit → no version bump, fresher last_seen.
    obs.emitted_at = 130;
    obs.observed_at = 130;
    s.record_peer_status(&obs).unwrap();
    let snaps = s.peer_session_snapshots(Some("proj"), 0).unwrap();
    assert_eq!(snaps[0].state_version, 1);
    assert_eq!(snaps[0].last_seen, 130);
    // Content change → version bump.
    obs.busy = false;
    obs.activity = String::new();
    obs.emitted_at = 160;
    obs.observed_at = 160;
    s.record_peer_status(&obs).unwrap();
    let snaps = s.peer_session_snapshots(Some("proj"), 0).unwrap();
    assert_eq!(snaps[0].state_version, 2);
    assert!(!snaps[0].busy);
}

/// register_or_reassert_session: alias hit reasserts the same canonical id;
/// a fresh harness id mints a new one.
#[test]
fn register_session_alias_hit_reasserts() {
    use crate::session::{Harness, SessionObservation};
    let s = Store::open_memory().unwrap();
    let obs = |sid: &str, ts: u64| SessionObservation {
        agent_slug: "claude".into(),
        agent_pubkey: "pk".into(),
        project: "proj".into(),
        host: "host".into(),
        rel_cwd: String::new(),
        harness: Harness::ClaudeCode,
        harness_session_id: Some(sid.into()),
        resume_id: None,
        tmux_pane: None,
        watch_pid: None,
        observed_at: ts,
    };
    let a = s.register_or_reassert_session(&obs("h1", 10)).unwrap();
    let a2 = s.register_or_reassert_session(&obs("h1", 20)).unwrap();
    assert_eq!(
        a.session_id, a2.session_id,
        "same harness id → same canonical id"
    );
    assert_eq!(
        a2.state_version, a.state_version,
        "identical reassert refreshes liveness without a public version bump"
    );
    assert_eq!(a2.last_seen, 20, "reassert refreshes liveness");
    let b = s.register_or_reassert_session(&obs("h2", 30)).unwrap();
    assert_ne!(
        a.session_id, b.session_id,
        "new harness id → new canonical id"
    );
}

/// all_live_local_snapshots feeds the heartbeat expiration re-arm: it must
/// return live sessions whose last_seen is fresh, drop stale ones, and drop
/// ended ones — otherwise live-but-idle sessions age off the relay.
#[test]
fn all_live_local_snapshots_filters_fresh_and_active() {
    use crate::session::{Harness, SessionObservation};
    let s = Store::open_memory().unwrap();
    let obs = SessionObservation {
        agent_slug: "claude".into(),
        agent_pubkey: "pk".into(),
        project: "proj".into(),
        host: "host".into(),
        rel_cwd: String::new(),
        harness: Harness::ClaudeCode,
        harness_session_id: Some("h1".into()),
        resume_id: None,
        tmux_pane: None,
        watch_pid: None,
        observed_at: 1000,
    };
    let snap = s.register_or_reassert_session(&obs).unwrap();
    s.heartbeat_session(snap.session_id.as_str(), 1000).ok();

    // Fresh window includes it; a window past its last_seen excludes it.
    assert_eq!(
        s.all_live_local_snapshots(910).unwrap().len(),
        1,
        "fresh → included"
    );
    assert!(
        s.all_live_local_snapshots(1001).unwrap().is_empty(),
        "stale → excluded"
    );

    // Ending the session drops it from the live set (lifecycle != active).
    s.end_session(snap.session_id.as_str(), 1000).ok();
    assert!(
        s.all_live_local_snapshots(910).unwrap().is_empty(),
        "ended → excluded even when last_seen is fresh"
    );
}

/// versioned distill guard: a stale base_version is rejected.
#[test]
fn apply_distill_result_rejects_stale_version() {
    use crate::session::{Harness, SessionObservation};
    let s = Store::open_memory().unwrap();
    let snap = s
        .register_or_reassert_session(&SessionObservation {
            agent_slug: "claude".into(),
            agent_pubkey: "pk".into(),
            project: "proj".into(),
            host: "host".into(),
            rel_cwd: String::new(),
            harness: Harness::ClaudeCode,
            harness_session_id: Some("h1".into()),
            resume_id: None,
            tmux_pane: None,
            watch_pid: None,
            observed_at: 10,
        })
        .unwrap();
    let turn = s.start_turn(snap.session_id.as_str(), 20).unwrap().unwrap();
    // Wrong base_version → rejected.
    assert!(s
        .apply_distill_result(
            turn.session_id.as_str(),
            turn.turn_id,
            turn.state_version + 99,
            "T",
            "A",
            30
        )
        .unwrap()
        .is_none());
    // Correct (turn_id, state_version) → applied.
    let applied = s
        .apply_distill_result(
            turn.session_id.as_str(),
            turn.turn_id,
            turn.state_version,
            "Distilled",
            "doing",
            30,
        )
        .unwrap();
    assert_eq!(applied.unwrap().title, "Distilled");
}

/// materialize_membership_snapshot replaces legacy group_members AND mirrors
/// into canonical membership when a project origin already exists.
#[test]
fn phase2_materialize_membership_snapshot_updates_both_tables() {
    let s = Store::open_memory().unwrap();
    // Seed a legacy stale member.
    s.upsert_group_member("proj", "stale", "member", 50)
        .unwrap();
    // Seed canonical origin.
    let pid = s
        .ensure_project_origin("nip29", "ri", "proj", "proj", 1)
        .unwrap();

    let members = vec![
        ("pk-a".to_string(), "member".to_string()),
        ("pk-b".to_string(), "admin".to_string()),
    ];
    s.materialize_membership_snapshot("proj", &members, "ri", 200)
        .unwrap();

    // Legacy table: stale gone, new members present.
    assert!(!s.is_group_member("proj", "stale").unwrap());
    assert!(s.is_group_member("proj", "pk-a").unwrap());
    assert!(s.is_group_member("proj", "pk-b").unwrap());

    // Canonical membership mirrored.
    assert_eq!(
        s.is_member_at(&pid, "pk-a", 300).unwrap(),
        MembershipDecision::Member {
            role: "member".into()
        }
    );
    assert_eq!(
        s.is_member_at(&pid, "pk-b", 300).unwrap(),
        MembershipDecision::Member {
            role: "admin".into()
        }
    );
}

/// materialize_membership_snapshot still updates legacy even without a canonical origin.
#[test]
fn phase2_materialize_membership_no_origin_still_updates_legacy() {
    let s = Store::open_memory().unwrap();
    let members = vec![("pk-x".to_string(), "member".to_string())];
    // No project_origins row → canonical mirror is a no-op, legacy still updates.
    s.materialize_membership_snapshot("unknown-proj", &members, "ri", 200)
        .unwrap();
    assert!(s.is_group_member("unknown-proj", "pk-x").unwrap());
}

// ── Phase 6 dual-write tests ──────────────────────────────────────────────

// ── Phase 7 tests ─────────────────────────────────────────────────────────
