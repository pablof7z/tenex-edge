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

#[test]
fn session_roundtrip_and_death() {
    let s = Store::open_memory().unwrap();
    s.upsert_session(&sample_session("sess-1")).unwrap();
    assert_eq!(
        s.get_session("sess-1").unwrap().unwrap(),
        sample_session("sess-1")
    );
    assert_eq!(s.list_alive_sessions().unwrap().len(), 1);
    s.mark_session_dead("sess-1").unwrap();
    assert!(s.list_alive_sessions().unwrap().is_empty());
    assert!(!s.get_session("sess-1").unwrap().unwrap().alive);
}

/// Bug C (agent-scoped sender resolution): the latest-alive fallback must be
/// scoped to the invoking agent, not the most-recently-active session of ANY
/// agent in the project. Otherwise a `claude` send is recorded as `opencode`
/// merely because opencode was the latest-active session.
#[test]
fn latest_alive_session_is_agent_scoped() {
    let s = Store::open_memory().unwrap();
    let mut claude = sample_session("sess-claude");
    claude.agent_slug = "claude".into();
    claude.agent_pubkey = "pk-claude".into();
    claude.created_at = 100;
    s.upsert_session(&claude).unwrap();

    let mut opencode = sample_session("sess-opencode");
    opencode.agent_slug = "opencode".into();
    opencode.agent_pubkey = "pk-opencode".into();
    opencode.created_at = 200; // more recently active
    s.upsert_session(&opencode).unwrap();

    // Agent-agnostic lookup returns opencode (the latest active) — the BUG.
    assert_eq!(
        s.latest_alive_session_for_project("proj")
            .unwrap()
            .unwrap()
            .agent_slug,
        "opencode"
    );
    // Agent-scoped lookup honors the invoking agent.
    assert_eq!(
        s.latest_alive_session_for_agent_in_project("claude", "proj")
            .unwrap()
            .unwrap()
            .agent_slug,
        "claude"
    );
    assert_eq!(
        s.latest_alive_session_for_agent_in_project("opencode", "proj")
            .unwrap()
            .unwrap()
            .agent_slug,
        "opencode"
    );
    // No alive session for an unknown agent.
    assert!(s
        .latest_alive_session_for_agent_in_project("codex", "proj")
        .unwrap()
        .is_none());
}

#[test]
fn resolve_with_project_scope_prefers_matching_presence() {
    let s = Store::open_memory().unwrap();
    s.upsert_peer_session(
        "sess-x",
        "pk-from-presence",
        "reviewer",
        "proj",
        "host",
        "",
        1,
    )
    .unwrap();
    assert_eq!(
        s.resolve_agent_pubkey("reviewer", Some("proj"))
            .unwrap()
            .as_deref(),
        Some("pk-from-presence")
    );
    s.upsert_profile("pk-from-profile", "reviewer", "host", false, 2)
        .unwrap();
    assert_eq!(
        s.resolve_agent_pubkey("reviewer", Some("proj"))
            .unwrap()
            .as_deref(),
        Some("pk-from-presence")
    );
    assert_eq!(
        s.resolve_agent_pubkey("reviewer", Some("other"))
            .unwrap()
            .as_deref(),
        None
    );
    assert_eq!(
        s.resolve_agent_pubkey("reviewer", None).unwrap().as_deref(),
        Some("pk-from-profile")
    );
}

#[test]
fn peer_freshness_and_prune() {
    let s = Store::open_memory().unwrap();
    s.upsert_peer_session("old", "pk1", "stale", "proj", "h", "", 100)
        .unwrap();
    s.upsert_peer_session("new", "pk2", "live", "proj", "h", "", 1000)
        .unwrap();
    // since=500 → only the fresh one is "live"
    let live = s.list_peer_sessions(Some("proj"), 500).unwrap();
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].slug, "live");
    // since=0 → both
    assert_eq!(s.list_peer_sessions(Some("proj"), 0).unwrap().len(), 2);
    // prune removes the stale one
    assert_eq!(s.prune_peer_sessions(500).unwrap(), 1);
    assert_eq!(s.list_peer_sessions(Some("proj"), 0).unwrap().len(), 1);
}

#[test]
fn rel_cwd_persists_on_peer_and_own_sessions() {
    let s = Store::open_memory().unwrap();
    // Peer session learns rel_cwd from presence.
    s.upsert_peer_session("p1", "pk", "rev", "proj", "tower", "worktree2", 1_000)
        .unwrap();
    let peers = s.list_peer_sessions(Some("proj"), 0).unwrap();
    assert_eq!(peers[0].rel_cwd, "worktree2");
    // Updating keeps the latest rel_cwd.
    s.upsert_peer_session("p1", "pk", "rev", "proj", "tower", "sub/dir", 1_001)
        .unwrap();
    assert_eq!(
        s.list_peer_sessions(Some("proj"), 0).unwrap()[0].rel_cwd,
        "sub/dir"
    );

    // Own session stores + reads back rel_cwd (needed by reconcile).
    s.upsert_session(&sample_session("mine")).unwrap();
    let mut rec = sample_session("mine");
    rec.rel_cwd = "worktree1".into();
    s.upsert_session(&rec).unwrap();
    assert_eq!(s.get_session("mine").unwrap().unwrap().rel_cwd, "worktree1");
}

#[test]
fn rel_cwd_migration_is_idempotent_on_reopen() {
    // Opening an on-disk db twice must not fail on the guarded ALTER TABLE
    // (the column already exists the second time).
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    {
        let s = Store::open(&path).unwrap();
        let mut rec = sample_session("m");
        rec.rel_cwd = "wt".into();
        s.upsert_session(&rec).unwrap();
    }
    let s2 = Store::open(&path).unwrap();
    assert_eq!(s2.get_session("m").unwrap().unwrap().rel_cwd, "wt");
}

#[test]
fn session_prefix_lookup() {
    let s = Store::open_memory().unwrap();
    s.upsert_peer_session("abcdef123456", "pk", "coder", "proj", "host", "", 1)
        .unwrap();
    let found = s.find_peer_session_by_prefix("abcdef").unwrap().unwrap();
    assert_eq!(found.pubkey, "pk");
    assert!(s.find_peer_session_by_prefix("zzzz").unwrap().is_none());
}

#[test]
fn turn_delta_peer_sessions_can_be_project_scoped() {
    let s = Store::open_memory().unwrap();
    s.upsert_peer_session("sess-a", "pk-a", "same", "current", "host", "", 100)
        .unwrap();
    s.upsert_peer_session("sess-b", "pk-b", "other", "elsewhere", "host", "", 100)
        .unwrap();

    let scoped = s.list_new_peer_sessions(50, 50, Some("current")).unwrap();
    assert_eq!(scoped.len(), 1);
    assert_eq!(scoped[0].session_id, "sess-a");

    let all = s.list_new_peer_sessions(50, 50, None).unwrap();
    assert_eq!(all.len(), 2);
}

/// A session that registers, starts a turn, then ends a turn surfaces in
/// `status_delta_since` as Changed; a freshly registered one as Appeared; an
/// ended one as Gone. Project-scoped + self-excluded.
#[test]
fn status_delta_since_classifies_appeared_changed_gone() {
    use crate::session::{DeltaKind, Harness, SessionObservation};
    let s = Store::open_memory().unwrap();
    let mk = |slug: &str, pk: &str, proj: &str, ts: u64| SessionObservation {
        agent_slug: slug.into(),
        agent_pubkey: pk.into(),
        project: proj.into(),
        host: "host".into(),
        rel_cwd: String::new(),
        harness: Harness::ClaudeCode,
        harness_session_id: Some(format!("h-{slug}")),
        resume_id: None,
        tmux_pane: None,
        watch_pid: None,
        observed_at: ts,
    };
    // Registered before the cursor → not "appeared", but a turn change after.
    let a = s
        .register_or_reassert_session(&mk("alpha", "pk-a", "proj", 100))
        .unwrap();
    // Registered AFTER the cursor → appeared.
    let now = 200u64;
    let since = 150u64;
    let b = s
        .register_or_reassert_session(&mk("bravo", "pk-b", "proj", 160))
        .unwrap();
    // Different project → excluded.
    let _ = s
        .register_or_reassert_session(&mk("gamma", "pk-c", "other", 160))
        .unwrap();
    // alpha changes after the cursor.
    s.start_turn(a.session_id.as_str(), 170).unwrap();

    let delta = s
        .status_delta_since("proj", since, now, Some(b.session_id.as_str()))
        .unwrap();
    // bravo is excluded; alpha must be present as Changed.
    assert!(delta
        .iter()
        .any(|d| d.snapshot.session_id == a.session_id && d.kind == DeltaKind::Changed));
    assert!(delta.iter().all(|d| d.snapshot.project == "proj"));

    // End alpha's session → it surfaces as Gone.
    s.end_session(a.session_id.as_str(), 180).unwrap();
    let delta2 = s.status_delta_since("proj", since, now, None).unwrap();
    assert!(delta2
        .iter()
        .any(|d| d.snapshot.session_id == a.session_id && d.kind == DeltaKind::Gone));
}

/// A local session whose own kind:30315 round-trips back from the relay into
/// `peer_session_state` MUST surface in the delta exactly ONCE. Before the
/// dedup, the local row and its peer echo were both emitted, producing the
/// duplicated (mirrored) lines in the turn-start fabric block.
#[test]
fn status_delta_since_dedups_local_session_peer_echo() {
    use crate::session::{Harness, PeerStatusObservation, SessionObservation};
    let s = Store::open_memory().unwrap();
    let _local = s
        .register_or_reassert_session(&SessionObservation {
            agent_slug: "alpha".into(),
            agent_pubkey: "pk-a".into(),
            project: "proj".into(),
            host: "host".into(),
            rel_cwd: String::new(),
            harness: Harness::ClaudeCode,
            harness_session_id: Some("h-alpha".into()),
            resume_id: None,
            tmux_pane: None,
            watch_pid: None,
            observed_at: 160,
        })
        .unwrap();
    // The same agent's status, observed back off the relay as a peer echo.
    // Keyed by (pubkey, project) — no session_id on the peer row.
    s.record_peer_status(&PeerStatusObservation {
        agent_pubkey: "pk-a".into(),
        agent_slug: "alpha".into(),
        project: "proj".into(),
        host: "host".into(),
        rel_cwd: String::new(),
        title: String::new(),
        activity: String::new(),
        busy: false,
        emitted_at: 165,
        observed_at: 165,
    })
    .unwrap();

    let delta = s.status_delta_since("proj", 150, 200, None).unwrap();
    // The local row surfaces; the peer echo (same pubkey) is deduped out.
    let hits = delta
        .iter()
        .filter(|d| d.snapshot.agent_pubkey == "pk-a")
        .count();
    assert_eq!(
        hits, 1,
        "local session + its own peer echo must dedup to one (keyed by pubkey)"
    );
}

/// A session is never told about its own status: even when its own kind:30315
/// has round-tripped into `peer_session_state`, passing the session as
/// `exclude` drops BOTH the local row and the peer echo.
#[test]
fn status_delta_since_excludes_self_even_with_peer_echo() {
    use crate::session::{Harness, PeerStatusObservation, SessionObservation};
    let s = Store::open_memory().unwrap();
    let me = s
        .register_or_reassert_session(&SessionObservation {
            agent_slug: "me".into(),
            agent_pubkey: "pk-me".into(),
            project: "proj".into(),
            host: "host".into(),
            rel_cwd: String::new(),
            harness: Harness::ClaudeCode,
            harness_session_id: Some("h-me".into()),
            resume_id: None,
            tmux_pane: None,
            watch_pid: None,
            observed_at: 160,
        })
        .unwrap();
    s.record_peer_status(&PeerStatusObservation {
        agent_pubkey: "pk-me".into(),
        agent_slug: "me".into(),
        project: "proj".into(),
        host: "host".into(),
        rel_cwd: String::new(),
        title: String::new(),
        activity: String::new(),
        busy: false,
        emitted_at: 165,
        observed_at: 165,
    })
    .unwrap();

    let delta = s
        .status_delta_since("proj", 150, 200, Some(me.session_id.as_str()))
        .unwrap();
    // The local row is excluded by session_id; the peer echo is deduped by
    // pubkey since "pk-me" appears in local_pubkeys.
    assert!(
        delta.iter().all(|d| d.snapshot.agent_pubkey != "pk-me"),
        "a session must never see its own status (local row or peer echo)"
    );
}

/// A still-`active` session whose heartbeats stopped (no event for > TTL)
/// MUST surface as `Gone` (liveness expired within the window) — a session
/// that drops off the relay stays reportable as gone, never silently lingers.
#[test]
fn status_delta_since_reports_expired_session_as_gone() {
    use crate::session::{DeltaKind, Harness, SessionObservation};
    let s = Store::open_memory().unwrap();
    let obs = SessionObservation {
        agent_slug: "ghost".into(),
        agent_pubkey: "pk-ghost".into(),
        project: "proj".into(),
        host: "host".into(),
        rel_cwd: String::new(),
        harness: Harness::ClaudeCode,
        harness_session_id: Some("h-ghost".into()),
        resume_id: None,
        tmux_pane: None,
        watch_pid: None,
        observed_at: 100,
    };
    // Registered + last seen at t=100, then never heard from again.
    let ghost = s.register_or_reassert_session(&obs).unwrap();
    // `now` is far past last_seen + STATUS_TTL_SECS; the cursor is between the
    // last sighting and now, so the expiry falls inside the window.
    let now = 100 + crate::domain::STATUS_TTL_SECS + 200;
    let since = 100 + crate::domain::STATUS_TTL_SECS / 2;
    let delta = s.status_delta_since("proj", since, now, None).unwrap();
    let item = delta
        .iter()
        .find(|d| d.snapshot.session_id == ghost.session_id)
        .expect("expired session must still surface in the delta");
    assert_eq!(item.kind, DeltaKind::Gone, "expired session must be Gone");
    assert!(
        !item.derived.liveness.is_live(),
        "an expired session is never live"
    );
}
