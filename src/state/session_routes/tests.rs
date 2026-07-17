use super::*;

fn running(store: &Store) -> (u64, Session) {
    let generation = store
        .reserve_session(&RegisterSession {
            pubkey: "pk".into(),
            harness: "grok".into(),
            agent_slug: "grok".into(),
            channel_h: "root".into(),
            child_pid: None,
            transcript_path: None,
            now: 1,
        })
        .unwrap();
    (generation, store.get_session("pk").unwrap().unwrap())
}

#[test]
fn route_affinity_survives_standing_removal() {
    let store = Store::open_memory().unwrap();
    store.grant_session_route("pk", "room", 1).unwrap();
    store
        .conn
        .execute(
            "INSERT INTO session_standing VALUES ('pk','room','retained',10,1,2,2)",
            [],
        )
        .unwrap();
    store
        .mark_session_standing_absent_if_epoch("pk", "room", StandingState::Retained, 1, 2, 10)
        .unwrap();

    assert!(store.has_session_route("pk", "room").unwrap());
}

#[test]
fn confirmed_runtime_join_is_retained_atomically_on_stop() {
    let store = Store::open_memory().unwrap();
    let (generation, session) = running(&store);
    assert_eq!(
        store
            .commit_confirmed_session_admission(
                "pk",
                "joined",
                generation,
                session.lifecycle_epoch,
                2,
            )
            .unwrap(),
        ConfirmedAdmissionCommit::Committed
    );
    store
        .mark_runtime_stopped_if_generation("pk", generation, StopReason::Crash, 10)
        .unwrap();

    let joined = store.get_session_standing("pk", "joined").unwrap().unwrap();
    assert_eq!(joined.state, StandingState::Retained);
    assert_eq!(joined.retain_until, 10 + STOPPED_STANDING_RETENTION_SECS);
}

#[test]
fn stale_confirmed_admission_is_persisted_due_for_cleanup() {
    let store = Store::open_memory().unwrap();
    let (generation, session) = running(&store);
    store
        .mark_runtime_stopped_if_generation("pk", generation, StopReason::Crash, 10)
        .unwrap();

    let result = store
        .commit_confirmed_session_admission("pk", "joined", generation, session.lifecycle_epoch, 11)
        .unwrap();
    let ConfirmedAdmissionCommit::CleanupDue(due) = result else {
        panic!("expected durable cleanup")
    };
    assert_eq!(due.state, StandingState::Retained);
    assert_eq!(due.retain_until, 11);
    assert_eq!(due.session_lifecycle_epoch, session.lifecycle_epoch);
    assert!(!store.has_session_route("pk", "joined").unwrap());
}

#[test]
fn compensation_fallback_recognizes_a_committed_admission() {
    let store = Store::open_memory().unwrap();
    let (generation, session) = running(&store);
    store
        .commit_confirmed_session_admission("pk", "joined", generation, session.lifecycle_epoch, 2)
        .unwrap();

    assert_eq!(
        store
            .schedule_confirmed_admission_cleanup(
                "pk",
                "joined",
                generation,
                session.lifecycle_epoch,
                3,
            )
            .unwrap(),
        ConfirmedAdmissionCommit::Committed
    );
}

#[test]
fn compensation_fallback_persists_due_after_primary_write_error() {
    let store = Store::open_memory().unwrap();
    let (generation, session) = running(&store);
    store
        .conn
        .execute_batch(
            "CREATE TRIGGER reject_join_route BEFORE INSERT ON session_channels
             WHEN NEW.channel_h='joined'
             BEGIN SELECT RAISE(FAIL, 'forced route failure'); END;",
        )
        .unwrap();
    assert!(store
        .commit_confirmed_session_admission("pk", "joined", generation, session.lifecycle_epoch, 2,)
        .is_err());

    let result = store
        .schedule_confirmed_admission_cleanup(
            "pk",
            "joined",
            generation,
            session.lifecycle_epoch,
            3,
        )
        .unwrap();
    let ConfirmedAdmissionCommit::CleanupDue(due) = result else {
        panic!("expected durable cleanup")
    };
    assert_eq!(due.retain_until, 3);
    assert_eq!(due.state, StandingState::Retained);
}
