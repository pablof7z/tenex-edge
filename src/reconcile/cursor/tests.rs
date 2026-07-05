use super::*;

fn seed() -> CursorSeed {
    CursorSeed {
        session_id: "s1".into(),
        seen_cursor: 10,
    }
}

fn fact(observed_cursor: u64, at: u64, working: bool) -> InputFact {
    InputFact::TurnCheckRequested {
        session_id: "s1".into(),
        observed_cursor,
        working,
        at,
    }
}

#[test]
fn first_matching_turn_check_advances_cursor() {
    let mut r = CursorReconciler::new();
    let out = r.request(seed(), fact(10, 20, true)).unwrap();
    r.assert_oracle().unwrap();

    assert_eq!(
        out.effects,
        vec![CursorEffect::Advance {
            session_id: "s1".into(),
            from: 10,
            to: 20,
            delta_since: 10,
        }]
    );
    let why = r.explain_cursor("s1").unwrap();
    assert_eq!(why.resource_key, "cursor/s1");
    assert!(why
        .input_causes
        .iter()
        .any(|c| c == "cursor/s1/observed_cursor"));
}

#[test]
fn stale_parallel_observation_gets_no_frame_after_advance() {
    let mut r = CursorReconciler::new();
    r.request(seed(), fact(10, 20, true)).unwrap();

    let stale = r.request(seed(), fact(10, 21, true)).unwrap();

    assert_eq!(stale.effects, vec![CursorEffect::NoFrame]);
    assert_eq!(r.state_rows()[0].cursor, 20);
    assert_eq!(r.state_rows()[0].last_frame, "NoFrame");
}

#[test]
fn preview_does_not_mutate_cursor_state() {
    let mut r = CursorReconciler::new();
    let preview = r.preview_request(seed(), &fact(10, 20, true)).unwrap();

    assert_eq!(r.revision(), 0);
    assert!(r.state_rows().is_empty());
    assert_eq!(preview.result.resource_plan.commands().len(), 1);
}
