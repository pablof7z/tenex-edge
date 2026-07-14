use super::*;

#[tokio::test]
async fn failed_offline_resume_restores_existing_aliases() {
    let state = DaemonState::new_for_test().await;
    let session_id = state
        .with_store(|store| {
            store.register_session(&crate::state::RegisterSession {
                harness: "codex".to_string(),
                external_id_kind: "resume".to_string(),
                external_id: "native".to_string(),
                agent_pubkey: "pubkey".to_string(),
                agent_slug: "codex".to_string(),
                channel_h: "root".to_string(),
                child_pid: None,
                transcript_path: None,
                resume_id: "native".to_string(),
                now: 10,
            })
        })
        .unwrap();
    state
        .with_store(|store| store.mark_dead(&session_id))
        .unwrap();

    {
        let (resolved, _, _, guard) = resolve_session_id(
            &state,
            "codex",
            Some("%fresh"),
            None,
            Some("native"),
            None,
            false,
            20,
        )
        .unwrap();
        assert_eq!(resolved, session_id);
        record_secondary_aliases(
            &guard,
            "codex",
            &session_id,
            Some("%fresh"),
            None,
            Some("native"),
            None,
            "root",
            std::path::Path::new("/repo"),
            "root",
            20,
        );
        // A later allocation failure returns while the guard is armed.
    }

    state
        .with_store(|store| {
            let resume = store
                .aliases_for_external_id(Some("codex"), "resume", "native")?
                .into_iter()
                .next()
                .expect("original resume alias remains");
            assert_eq!(resume.session_id, session_id);
            assert_eq!(resume.created_at, 10);
            assert!(store
                .aliases_for_external_id(Some("codex"), "pty_session", "%fresh")?
                .is_empty());
            Ok::<_, anyhow::Error>(())
        })
        .unwrap();
}

#[tokio::test]
async fn failed_attempt_never_overwrites_a_newer_alias_owner() {
    let state = DaemonState::new_for_test().await;
    let guard = AliasRollbackGuard::new(&state, "codex");
    state
        .with_store(|store| {
            store.put_alias("codex", "pty_session", "%captured", "old", 10)?;
            store.put_alias("codex", "pty_session", "%captured", "winner", 20)?;
            store.put_alias_provisional(
                "codex",
                "pty_session",
                "%captured",
                "winner",
                20,
                &guard.owner,
            )?;

            store.put_alias_provisional(
                "codex",
                "pty_session",
                "%newer",
                "failed",
                20,
                &guard.owner,
            )?;
            store.put_alias("codex", "pty_session", "%newer", "newer-winner", 20)?;
            Ok::<_, anyhow::Error>(())
        })
        .unwrap();

    drop(guard);

    state
        .with_store(|store| {
            assert_eq!(
                store
                    .resolve_session_by_alias("codex", "pty_session", "%captured")?
                    .as_deref(),
                Some("winner")
            );
            assert_eq!(
                store
                    .resolve_session_by_alias("codex", "pty_session", "%newer")?
                    .as_deref(),
                Some("newer-winner")
            );
            Ok::<_, anyhow::Error>(())
        })
        .unwrap();
}

#[tokio::test]
async fn overlapping_attempts_settle_without_reviving_aborted_owners() {
    let state = DaemonState::new_for_test().await;
    let first = AliasRollbackGuard::new(&state, "codex");
    let second = AliasRollbackGuard::new(&state, "codex");
    state
        .with_store(|store| {
            store.put_alias("codex", "resume", "shared", "original", 1)?;
            store.put_alias_provisional("codex", "resume", "shared", "first", 2, &first.owner)?;
            store.put_alias_provisional("codex", "resume", "shared", "second", 3, &second.owner)?;
            Ok::<_, anyhow::Error>(())
        })
        .unwrap();

    drop(first);
    assert_eq!(
        state
            .with_store(|store| store.resolve_session_by_alias("codex", "resume", "shared"))
            .unwrap()
            .as_deref(),
        Some("second")
    );
    drop(second);
    assert_eq!(
        state
            .with_store(|store| store.resolve_session_by_alias("codex", "resume", "shared"))
            .unwrap()
            .as_deref(),
        Some("original")
    );
}

#[tokio::test]
async fn committed_shadowed_attempt_survives_newer_abort() {
    let state = DaemonState::new_for_test().await;
    let mut first = AliasRollbackGuard::new(&state, "codex");
    let second = AliasRollbackGuard::new(&state, "codex");
    state
        .with_store(|store| {
            store.put_alias_provisional("codex", "resume", "shared", "first", 2, &first.owner)?;
            store.put_alias_provisional("codex", "resume", "shared", "second", 3, &second.owner)?;
            Ok::<_, anyhow::Error>(())
        })
        .unwrap();

    first.disarm();
    drop(second);

    assert_eq!(
        state
            .with_store(|store| store.resolve_session_by_alias("codex", "resume", "shared"))
            .unwrap()
            .as_deref(),
        Some("first")
    );
}

#[tokio::test]
async fn deliberate_alias_clear_cannot_be_revived_by_failed_attempt() {
    let state = DaemonState::new_for_test().await;
    let guard = AliasRollbackGuard::new(&state, "codex");
    state
        .with_store(|store| {
            store.put_alias("codex", "pty_session", "%dead", "original", 1)?;
            store.put_alias_provisional(
                "codex",
                "pty_session",
                "%dead",
                "failed",
                2,
                &guard.owner,
            )?;
            store.clear_alias_kind("failed", "pty_session")?;
            Ok::<_, anyhow::Error>(())
        })
        .unwrap();

    drop(guard);

    assert!(state
        .with_store(|store| {
            store.aliases_for_external_id(Some("codex"), "pty_session", "%dead")
        })
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn clearing_shadowed_target_tombstones_rollback_frame() {
    let state = DaemonState::new_for_test().await;
    let guard = AliasRollbackGuard::new(&state, "codex");
    state
        .with_store(|store| {
            store.put_alias("codex", "pty_session", "%dead", "retired", 1)?;
            store.put_alias_provisional(
                "codex",
                "pty_session",
                "%dead",
                "new-attempt",
                2,
                &guard.owner,
            )?;
            store.clear_alias_kind("retired", "pty_session")?;
            Ok::<_, anyhow::Error>(())
        })
        .unwrap();

    drop(guard);

    assert!(state
        .with_store(|store| {
            store.aliases_for_external_id(Some("codex"), "pty_session", "%dead")
        })
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn stale_target_cannot_be_validated_through_an_untyped_locator() {
    let state = DaemonState::new_for_test().await;
    state
        .with_store(|store| {
            store.register_session(&crate::state::RegisterSession {
                harness: "codex".to_string(),
                external_id_kind: "watch_pid".to_string(),
                external_id: "stale-target".to_string(),
                agent_pubkey: "real-pubkey".to_string(),
                agent_slug: "codex".to_string(),
                channel_h: "root".to_string(),
                child_pid: None,
                transcript_path: None,
                resume_id: String::new(),
                now: 1,
            })?;
            store.put_alias("codex", "resume", "token", "stale-target", 2)?;

            let selected = select_session_id(store, "codex", "resume", "token", false)?;
            assert_ne!(selected, "stale-target");
            Ok::<_, anyhow::Error>(())
        })
        .unwrap();
}
