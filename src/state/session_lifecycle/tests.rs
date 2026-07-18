use super::*;

fn running(store: &Store) -> Session {
    store.get_session("pk").unwrap().unwrap()
}

fn seed() -> Store {
    let store = Store::open_memory().unwrap();
    let registration = RegisterSession {
        pubkey: "pk".into(),
        observed_harness: "grok".into(),
        agent_slug: "grok".into(),
        channel_h: "room".into(),
        child_pid: Some(42),
        transcript_path: None,
        now: 1,
    };
    store
        .reserve_session_with_facts(
            &registration,
            &AdmittedRuntimeFacts {
                observed_harness: "grok".into(),
                claimed_harness: String::new(),
                bundle: "grok-pty".into(),
                transport: "pty".into(),
                endpoint_provenance: "launch".into(),
            },
        )
        .unwrap();
    store
}

#[test]
fn detach_while_working_arms_only_when_the_turn_ends() {
    let store = seed();
    let generation = running(&store).runtime_generation;
    assert!(store
        .apply_session_presentation_edge("pk", generation, 1, PresentationState::Headed, 5)
        .unwrap());
    assert!(store
        .apply_session_turn_started("pk", generation, 10, None)
        .unwrap());
    assert!(store
        .apply_session_presentation_edge("pk", generation, 2, PresentationState::Headless, 20)
        .unwrap());
    assert_eq!(running(&store).idle_deadline, 0);

    assert!(store
        .apply_session_turn_ended("pk", generation, 30)
        .unwrap());
    assert_eq!(
        running(&store).idle_deadline,
        30 + HEADLESS_IDLE_TIMEOUT_SECS
    );
    assert!(store
        .list_due_idle_evictions(29 + HEADLESS_IDLE_TIMEOUT_SECS)
        .unwrap()
        .is_empty());
    assert_eq!(
        store
            .list_due_idle_evictions(30 + HEADLESS_IDLE_TIMEOUT_SECS)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn reattach_cancels_deadline_and_stale_edges_are_ignored() {
    let store = seed();
    let session = running(&store);
    store
        .apply_session_presentation_edge(
            "pk",
            session.runtime_generation,
            2,
            PresentationState::Headless,
            10,
        )
        .unwrap();
    assert!(store
        .apply_session_presentation_edge(
            "pk",
            session.runtime_generation,
            3,
            PresentationState::Headed,
            20,
        )
        .unwrap());
    assert!(!store
        .apply_session_presentation_edge(
            "pk",
            session.runtime_generation,
            2,
            PresentationState::Headless,
            30,
        )
        .unwrap());
    let current = running(&store);
    assert_eq!(current.presentation_state, PresentationState::Headed);
    assert_eq!(current.idle_deadline, 0);
}

#[test]
fn pending_inbox_fences_idle_eviction() {
    let store = seed();
    let session = running(&store);
    store
        .apply_session_presentation_edge(
            "pk",
            session.runtime_generation,
            1,
            PresentationState::Headless,
            10,
        )
        .unwrap();
    store
        .enqueue_inbox("event", "pk", "human", "room", "hello", 20)
        .unwrap();
    let due = 10 + HEADLESS_IDLE_TIMEOUT_SECS;
    assert_eq!(running(&store).idle_deadline, 0);
    assert!(store.list_due_idle_evictions(due).unwrap().is_empty());
    assert!(store
        .reserve_due_idle_eviction(
            "pk",
            session.runtime_generation,
            session.lifecycle_epoch,
            1,
            due,
        )
        .unwrap()
        .is_none());
}

#[test]
fn initial_epoch_zero_snapshot_is_applied_once() {
    let store = seed();
    let session = running(&store);
    assert!(store
        .apply_session_presentation_edge(
            "pk",
            session.runtime_generation,
            0,
            PresentationState::Headless,
            10,
        )
        .unwrap());
    assert!(!store
        .apply_session_presentation_edge(
            "pk",
            session.runtime_generation,
            0,
            PresentationState::Headed,
            20,
        )
        .unwrap());
    assert_eq!(
        running(&store).presentation_state,
        PresentationState::Headless
    );
}

#[test]
fn stopping_epoch_fences_attach_and_finalize_races() {
    let store = seed();
    let initial = running(&store);
    store
        .apply_session_presentation_edge(
            "pk",
            initial.runtime_generation,
            1,
            PresentationState::Headless,
            10,
        )
        .unwrap();
    let stopping = store
        .reserve_due_idle_eviction(
            "pk",
            initial.runtime_generation,
            initial.lifecycle_epoch,
            1,
            10 + HEADLESS_IDLE_TIMEOUT_SECS,
        )
        .unwrap()
        .unwrap();
    assert!(store
        .cancel_idle_eviction_on_presentation_change(
            "pk",
            initial.runtime_generation,
            stopping.lifecycle_epoch,
            2,
            PresentationState::Headed,
            700,
        )
        .unwrap());
    assert!(store
        .finalize_runtime_stopped_if_epoch(
            "pk",
            initial.runtime_generation,
            stopping.lifecycle_epoch,
            StopReason::IdleEvicted,
            stopping.stopped_at,
        )
        .unwrap()
        .is_none());
    assert!(running(&store).is_running());
}

#[test]
fn attach_detach_race_returns_to_running_with_a_fresh_deadline() {
    let store = seed();
    let initial = running(&store);
    store
        .apply_session_presentation_edge(
            "pk",
            initial.runtime_generation,
            1,
            PresentationState::Headless,
            10,
        )
        .unwrap();
    let stopping = store
        .reserve_due_idle_eviction(
            "pk",
            initial.runtime_generation,
            initial.lifecycle_epoch,
            1,
            10 + HEADLESS_IDLE_TIMEOUT_SECS,
        )
        .unwrap()
        .unwrap();
    assert!(store
        .cancel_idle_eviction_on_presentation_change(
            "pk",
            initial.runtime_generation,
            stopping.lifecycle_epoch,
            3,
            PresentationState::Headless,
            700,
        )
        .unwrap());
    let current = running(&store);
    assert_eq!(current.runtime_state, RuntimeState::Running);
    assert_eq!(current.presentation_state, PresentationState::Headless);
    assert_eq!(current.stopped_at, 0);
    assert_eq!(current.idle_deadline, 700 + HEADLESS_IDLE_TIMEOUT_SECS);
}

#[test]
fn unavailable_conditional_kill_unwinds_stopping_at_the_same_epoch() {
    let store = seed();
    let initial = running(&store);
    store
        .apply_session_presentation_edge(
            "pk",
            initial.runtime_generation,
            1,
            PresentationState::Headless,
            10,
        )
        .unwrap();
    let stopping = store
        .reserve_due_idle_eviction(
            "pk",
            initial.runtime_generation,
            initial.lifecycle_epoch,
            1,
            10 + HEADLESS_IDLE_TIMEOUT_SECS,
        )
        .unwrap()
        .unwrap();
    assert!(store
        .cancel_idle_eviction_on_presentation_change(
            "pk",
            initial.runtime_generation,
            stopping.lifecycle_epoch,
            1,
            PresentationState::Unavailable,
            700,
        )
        .unwrap());
    let current = running(&store);
    assert_eq!(current.runtime_state, RuntimeState::Running);
    assert_eq!(current.presentation_state, PresentationState::Unavailable);
    assert_eq!(current.idle_deadline, 0);
}

#[test]
fn finalized_stop_retains_confirmed_standing_for_one_hour() {
    let store = seed();
    let initial = running(&store);
    store
        .mark_session_standing_member_if_running("pk", "room", initial.lifecycle_epoch, 2)
        .unwrap()
        .unwrap();
    store
        .apply_session_presentation_edge(
            "pk",
            initial.runtime_generation,
            1,
            PresentationState::Headless,
            10,
        )
        .unwrap();
    let stopping = store
        .reserve_due_idle_eviction(
            "pk",
            initial.runtime_generation,
            initial.lifecycle_epoch,
            1,
            10 + HEADLESS_IDLE_TIMEOUT_SECS,
        )
        .unwrap()
        .unwrap();
    store
        .finalize_runtime_stopped_if_epoch(
            "pk",
            initial.runtime_generation,
            stopping.lifecycle_epoch,
            StopReason::IdleEvicted,
            stopping.stopped_at,
        )
        .unwrap()
        .unwrap();
    let standing = store.list_session_standing("pk").unwrap();
    assert_eq!(standing[0].state, StandingState::Retained);
    assert_eq!(
        standing[0].retain_until,
        stopping.stopped_at + STOPPED_STANDING_RETENTION_SECS
    );
}

#[test]
fn generic_engine_exit_cannot_steal_idle_eviction_ownership() {
    let store = seed();
    let initial = running(&store);
    store
        .apply_session_presentation_edge(
            "pk",
            initial.runtime_generation,
            1,
            PresentationState::Headless,
            10,
        )
        .unwrap();
    let reserved_at = 10 + HEADLESS_IDLE_TIMEOUT_SECS;
    let stopping = store
        .reserve_due_idle_eviction(
            "pk",
            initial.runtime_generation,
            initial.lifecycle_epoch,
            1,
            reserved_at,
        )
        .unwrap()
        .unwrap();
    assert_eq!(stopping.stopped_at, reserved_at);
    assert!(!store
        .mark_runtime_stopped_if_generation(
            "pk",
            initial.runtime_generation,
            StopReason::Crash,
            reserved_at + 1,
        )
        .unwrap());
    let stopped = store
        .finalize_runtime_stopped_if_epoch(
            "pk",
            initial.runtime_generation,
            stopping.lifecycle_epoch,
            StopReason::IdleEvicted,
            stopping.stopped_at,
        )
        .unwrap()
        .unwrap();
    assert_eq!(stopped.stop_reason, Some(StopReason::IdleEvicted));
    assert_eq!(stopped.stopped_at, reserved_at);
}
