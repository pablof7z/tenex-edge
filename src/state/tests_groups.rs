use super::*;

#[test]
fn opencode_reassert_with_echoed_canonical_id_reattaches_not_supersedes() {
    use crate::session::{Harness, SessionObservation};
    let s = Store::open_memory().unwrap();
    // session-start: opencode owns no native id (echo harness), so the daemon
    // mints the canonical id, anchored on pane + watched pid. No
    // harness_session_id / resume_id yet.
    let start = SessionObservation {
        agent_slug: "opencode".into(),
        agent_pubkey: "pk-oc".into(),
        project: "proj".into(),
        host: "host".into(),
        rel_cwd: String::new(),
        harness: Harness::Opencode,
        harness_session_id: None,
        resume_id: None,
        tmux_pane: Some("%0".into()),
        watch_pid: Some(70282),
        observed_at: 100,
    };
    let canonical = s
        .register_or_reassert_session(&start)
        .unwrap()
        .session_id
        .as_str()
        .to_string();

    // user-prompt-submit: the plugin echoes the canonical id back as the
    // harness session id, now knows opencode's resume token, and reports a
    // DIFFERENT pid (ancestor search). Pre-fix this missed the alias lookup and
    // fell through to the pane/pid supersede branch, minting a brand-new
    // session on every first turn.
    let reassert = SessionObservation {
        agent_slug: "opencode".into(),
        agent_pubkey: "pk-oc".into(),
        project: "proj".into(),
        host: "host".into(),
        rel_cwd: String::new(),
        harness: Harness::Opencode,
        harness_session_id: Some(canonical.clone()),
        resume_id: Some("ses_abc".into()),
        tmux_pane: Some("%0".into()),
        watch_pid: Some(99999),
        observed_at: 160,
    };
    let after = s
        .register_or_reassert_session(&reassert)
        .unwrap()
        .session_id
        .as_str()
        .to_string();
    assert_eq!(
        after, canonical,
        "reassert must reattach to the same canonical session, not mint a new one"
    );
    // No churn: exactly one session_state row exists (pre-fix the reassert
    // superseded into a second row, leaving one ended + one active).
    let rows: i64 = s
        .conn
        .query_row("SELECT COUNT(*) FROM session_state", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        rows, 1,
        "exactly one session_state row (no churn), got {rows}"
    );
}

#[test]
fn turn_check_due_gates_and_advances_cursor() {
    let s = Store::open_memory().unwrap();
    // Not in a turn → never due (avoids querying all history).
    assert_eq!(s.turn_check_due("sess", 1000, 60).unwrap(), None);

    // Turn starts at t=1000; first check at t=1000 is due, since=turn start.
    s.mark_turn_start("sess", 1000).unwrap();
    assert_eq!(s.turn_check_due("sess", 1000, 60).unwrap(), Some(1000));

    // Within the 60s floor of the last check → suppressed.
    assert_eq!(s.turn_check_due("sess", 1059, 60).unwrap(), None);

    // 60s elapsed → due again, since = the previous check time (1000).
    assert_eq!(s.turn_check_due("sess", 1060, 60).unwrap(), Some(1000));
    // Cursor advanced to 1060: the next window starts there.
    assert_eq!(s.turn_check_due("sess", 1130, 60).unwrap(), Some(1060));

    // A new turn resets the cursor → first check is due immediately again.
    s.mark_turn_start("sess", 2000).unwrap();
    assert_eq!(s.turn_check_due("sess", 2000, 60).unwrap(), Some(2000));

    // Turn end clears working/cursor → not in a turn → not due.
    s.mark_turn_end("sess").unwrap();
    assert_eq!(s.turn_check_due("sess", 3000, 60).unwrap(), None);
}

#[test]
fn owned_groups_roundtrip_and_idempotent() {
    let s = Store::open_memory().unwrap();
    assert!(!s.is_group_owned("proj").unwrap());
    s.mark_group_owned("proj", 100).unwrap();
    assert!(s.is_group_owned("proj").unwrap());
    // Re-marking is a no-op (keeps the original created_at), not an error.
    s.mark_group_owned("proj", 200).unwrap();
    assert!(s.is_group_owned("proj").unwrap());
    assert!(!s.is_group_owned("other").unwrap());
}

#[test]
fn group_parent_distinguishes_subgroup_from_project() {
    let s = Store::open_memory().unwrap();
    // Unknown group → no parent.
    assert_eq!(s.group_parent("unknown").unwrap(), None);
    // Top-level project (empty parent) → None.
    s.upsert_group_metadata("proj", "Proj", "", 100).unwrap();
    assert_eq!(s.group_parent("proj").unwrap(), None);
    // Subgroup with a parent → Some(parent).
    s.upsert_group_metadata("proj-room", "Room", "proj", 100)
        .unwrap();
    assert_eq!(s.group_parent("proj-room").unwrap(), Some("proj".into()));
}

#[test]
fn group_member_upsert_and_query() {
    let s = Store::open_memory().unwrap();
    assert!(!s.is_group_member("proj", "pk-a").unwrap());
    s.upsert_group_member("proj", "pk-a", "member", 100)
        .unwrap();
    assert!(s.is_group_member("proj", "pk-a").unwrap());
    // Membership is per (project, pubkey).
    assert!(!s.is_group_member("other", "pk-a").unwrap());
    assert!(!s.is_group_member("proj", "pk-b").unwrap());
    // Upsert is idempotent on the primary key.
    s.upsert_group_member("proj", "pk-a", "admin", 200).unwrap();
    assert!(s.is_group_member("proj", "pk-a").unwrap());
}

#[test]
fn replace_group_members_is_authoritative() {
    let s = Store::open_memory().unwrap();
    s.upsert_group_member("proj", "stale", "member", 100)
        .unwrap();
    // A relay 39002 snapshot replaces the whole set: 'stale' drops out.
    s.replace_group_members(
        "proj",
        &[
            ("pk-a".into(), "member".into()),
            ("pk-b".into(), "admin".into()),
        ],
        300,
    )
    .unwrap();
    assert!(!s.is_group_member("proj", "stale").unwrap());
    assert!(s.is_group_member("proj", "pk-a").unwrap());
    assert!(s.is_group_member("proj", "pk-b").unwrap());
    // Scoped to the project — a different group is untouched.
    s.upsert_group_member("other", "pk-x", "member", 100)
        .unwrap();
    s.replace_group_members("proj", &[], 400).unwrap();
    assert!(!s.is_group_member("proj", "pk-a").unwrap());
    assert!(s.is_group_member("other", "pk-x").unwrap());
}

// ── freeze tests (Phase-0 regression oracle) ─────────────────────────────

/// FREEZE B2: replace_group_members applied TWICE with the same snapshot is
/// idempotent — no duplicates, no stale survivors, and other projects are
/// unaffected. This extends the existing authoritative-replace test.
#[test]
fn freeze_replace_group_members_idempotent_re_apply() {
    let s = Store::open_memory().unwrap();
    let snapshot: Vec<(String, String)> = vec![
        ("pk-alpha".into(), "member".into()),
        ("pk-beta".into(), "admin".into()),
    ];

    // Seed a stale member that should vanish.
    s.upsert_group_member("proj", "pk-stale", "member", 50)
        .unwrap();

    // First apply.
    s.replace_group_members("proj", &snapshot, 200).unwrap();
    assert!(s.is_group_member("proj", "pk-alpha").unwrap());
    assert!(s.is_group_member("proj", "pk-beta").unwrap());
    assert!(!s.is_group_member("proj", "pk-stale").unwrap());

    // Identical second apply — observable membership must be unchanged.
    s.replace_group_members("proj", &snapshot, 300).unwrap();
    assert!(
        s.is_group_member("proj", "pk-alpha").unwrap(),
        "alpha still member after re-apply"
    );
    assert!(
        s.is_group_member("proj", "pk-beta").unwrap(),
        "beta still member after re-apply"
    );
    assert!(
        !s.is_group_member("proj", "pk-stale").unwrap(),
        "stale still absent after re-apply"
    );

    // A sibling project is completely unaffected by both applies.
    s.upsert_group_member("other-proj", "pk-other", "member", 100)
        .unwrap();
    s.replace_group_members("proj", &snapshot, 400).unwrap();
    assert!(
        s.is_group_member("other-proj", "pk-other").unwrap(),
        "sibling project untouched"
    );
    assert!(!s.is_group_member("other-proj", "pk-alpha").unwrap());
}
