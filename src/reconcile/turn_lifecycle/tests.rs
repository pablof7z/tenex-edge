use super::*;

fn seed() -> TurnProjectionSeed {
    TurnProjectionSeed {
        pubkey: "s1".into(),
        working: false,
        turn_started_at: 0,
        transcript_ref: None,
    }
}

fn applied(out: &TurnLifecycleOutcome) -> &TurnCommand {
    match out.effects.first().expect("one effect") {
        TurnEffect::Apply(cmd) => cmd,
    }
}

#[test]
fn turn_started_derives_session_projection_and_why() {
    let mut r = TurnLifecycleReconciler::new();

    let out = r
        .on_turn_started(seed(), 100, Some("/tmp/transcript.jsonl".into()))
        .unwrap();
    r.assert_oracle().unwrap();

    let cmd = applied(&out);
    assert_eq!(cmd.pubkey, "s1");
    assert!(cmd.working);
    assert_eq!(cmd.turn_started_at, 100);
    assert_eq!(cmd.transcript_ref.as_deref(), Some("/tmp/transcript.jsonl"));

    let why = r.explain_turn("s1").unwrap();
    assert_eq!(why.resource_key, "turn_lifecycle/s1");
    assert_eq!(why.last_kind, "Open");
    assert!(
        why.input_causes
            .iter()
            .any(|c| c == "turn_lifecycle/s1/turn_started"),
        "turn start should be an input cause: {:?}",
        why.input_causes
    );
}

#[test]
fn turn_ended_clears_working_and_start_timestamp() {
    let mut r = TurnLifecycleReconciler::new();
    r.on_turn_started(seed(), 100, None).unwrap();

    let out = r
        .on_turn_ended(
            TurnProjectionSeed {
                pubkey: "s1".into(),
                working: true,
                turn_started_at: 100,
                transcript_ref: None,
            },
            130,
        )
        .unwrap();

    let cmd = applied(&out);
    assert!(!cmd.working);
    assert_eq!(cmd.turn_started_at, 0);
    let why = r.explain_turn("s1").unwrap();
    assert_eq!(why.last_kind, "Replace");
    assert!(
        why.input_causes
            .iter()
            .any(|c| c == "turn_lifecycle/s1/turn_ended"),
        "turn end should be an input cause: {:?}",
        why.input_causes
    );
}

#[test]
fn preview_does_not_mutate_lifecycle_graph() {
    let mut r = TurnLifecycleReconciler::new();
    let rev = r.revision();

    let preview = r.preview_turn_started(seed(), 100, None).unwrap();

    assert_eq!(r.revision(), rev);
    assert_eq!(r.state_rows(), Vec::<TurnStateRow>::new());
    assert_eq!(preview.result.resource_plan.commands().len(), 1);
    let changed = preview.labels.labels_for(&preview.result.changed_inputs);
    assert!(
        changed
            .iter()
            .any(|c| c == "turn_lifecycle/s1/turn_started"),
        "preview should name the turn-start input: {changed:?}"
    );
}
