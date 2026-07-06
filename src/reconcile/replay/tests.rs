use super::*;

#[test]
fn rejects_mixed_surface_scripts() {
    let mut script = DataTransactionScript::new();
    script
        .step("one")
        .operation(InputFact::SubscriptionSync {
            snapshot: Default::default(),
            at: 1,
        })
        .commit();
    script
        .step("two")
        .operation(InputFact::StatusDrive(
            crate::reconcile::StatusDrive::Tick {
                session_id: "s1".into(),
                at: 1,
            },
        ))
        .commit();
    let err = replay_script(&script, false).unwrap_err();
    assert!(err.to_string().contains("mixes surfaces"));
}

#[test]
fn diagnosis_corpus_replay_fixtures_are_valid() {
    let leaked_close = replay_script_json(
        include_str!("../../../tests/fixtures/trellis_diagnosis/leaked-close.json"),
        false,
    )
    .unwrap();
    assert_eq!(leaked_close.surface, "subscriptions");
    assert_eq!(leaked_close.steps, 2);
    assert_eq!(
        leaked_close.resource_commands, 2,
        "first owner leaving must not close a shared subscription"
    );

    let false_republish = replay_script_json(
        include_str!("../../../tests/fixtures/trellis_diagnosis/false-republish.json"),
        false,
    )
    .unwrap();
    assert_eq!(false_republish.surface, "status");
    assert_eq!(false_republish.steps, 2);
    assert_eq!(
        false_republish.resource_commands, 1,
        "same-bucket unchanged tick must not republish status"
    );
}

#[test]
fn turn_lifecycle_replay_accepts_canonical_turn_facts() {
    let mut script = DataTransactionScript::new();
    script
        .step("start")
        .operation(InputFact::TurnStarted {
            session_id: "s1".into(),
            at: 100,
        })
        .commit();
    script
        .step("end")
        .operation(InputFact::TurnEnded {
            session_id: "s1".into(),
            at: 130,
        })
        .commit();

    let report = replay_script(&script, false).unwrap();
    assert_eq!(report.surface, "turn_lifecycle");
    assert_eq!(report.steps, 2);
    assert_eq!(report.resource_commands, 2);
}

#[test]
fn cursor_replay_accepts_canonical_turn_check_facts() {
    let mut script = DataTransactionScript::new();
    script
        .step("first")
        .operation(InputFact::TurnCheckRequested {
            session_id: "s1".into(),
            observed_cursor: 10,
            working: true,
            at: 20,
        })
        .commit();
    script
        .step("stale")
        .operation(InputFact::TurnCheckRequested {
            session_id: "s1".into(),
            observed_cursor: 10,
            working: true,
            at: 21,
        })
        .commit();

    let report = replay_script(&script, false).unwrap();
    assert_eq!(report.surface, "cursor");
    assert_eq!(report.steps, 2);
    assert_eq!(report.resource_commands, 2);
}

#[test]
fn outbox_replay_accepts_enqueue_and_publish_result_facts() {
    let mut script = DataTransactionScript::new();
    script
        .step("enqueue")
        .operation(InputFact::OutboxEnqueueApplied {
            local_id: 7,
            event_id: "ev7".into(),
            event_hash: "sha256:event".into(),
            source_surface: "status".into(),
            source_ref: "status/s1#tx:1".into(),
            at: 100,
        })
        .commit();
    script
        .step("accepted")
        .operation(InputFact::RelayPublishAccepted {
            local_id: 7,
            event_id: "ev7".into(),
            accepted: true,
            error: None,
            at: 120,
        })
        .commit();

    let report = replay_script(&script, false).unwrap();
    assert_eq!(report.surface, "outbox");
    assert_eq!(report.steps, 2);
    assert_eq!(report.resource_commands, 2);
}

#[test]
fn session_watch_replay_accepts_start_and_exit_facts() {
    let mut script = DataTransactionScript::new();
    script
        .step("start")
        .operation(InputFact::SessionStarted {
            session_id: "s1".into(),
            channel_h: Some("room".into()),
            agent_pubkey: Some("pk".into()),
            pid: Some(42),
            at: 100,
        })
        .commit();
    script
        .step("exit")
        .operation(InputFact::ProcessExited {
            session_id: Some("s1".into()),
            pid: 42,
            at: 120,
        })
        .commit();

    let report = replay_script(&script, false).unwrap();
    assert_eq!(report.surface, "session_watch");
    assert_eq!(report.steps, 2);
    assert_eq!(report.resource_commands, 2);
}
