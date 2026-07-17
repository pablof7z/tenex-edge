use super::*;

#[test]
fn standing_epoch_fences_expiry_after_reactivation() {
    let store = Store::open_memory().unwrap();
    store
        .reserve_session(&RegisterSession {
            pubkey: "pk".into(),
            harness: "grok".into(),
            agent_slug: "grok".into(),
            channel_h: "room".into(),
            child_pid: None,
            transcript_path: None,
            now: 1,
        })
        .unwrap();
    let running = store.get_session("pk").unwrap().unwrap();
    store
        .mark_session_standing_member_if_running("pk", "room", running.lifecycle_epoch, 2)
        .unwrap()
        .unwrap();
    store
        .mark_runtime_stopped_if_generation("pk", running.runtime_generation, StopReason::Crash, 10)
        .unwrap();
    let stopped = store.get_session("pk").unwrap().unwrap();
    let retained_rows = store.list_session_standing("pk").unwrap();
    let retained = retained_rows[0].standing_epoch;
    assert_eq!(store.list_due_retained_standing(3_609).unwrap(), []);
    assert_eq!(store.list_due_retained_standing(3_610).unwrap().len(), 1);

    store
        .reserve_session(&RegisterSession {
            pubkey: "pk".into(),
            harness: "grok".into(),
            agent_slug: "grok".into(),
            channel_h: "room".into(),
            child_pid: None,
            transcript_path: None,
            now: 90,
        })
        .unwrap();
    let resumed = store.get_session("pk").unwrap().unwrap();
    let member = store
        .mark_session_standing_member_if_running("pk", "room", resumed.lifecycle_epoch, 90)
        .unwrap()
        .unwrap();
    assert!(member > retained);
    assert!(!store
        .mark_session_standing_absent_if_epoch(
            "pk",
            "room",
            StandingState::Retained,
            retained,
            stopped.lifecycle_epoch,
            101,
        )
        .unwrap());
    assert_eq!(
        store
            .get_session_standing("pk", "room")
            .unwrap()
            .unwrap()
            .state,
        StandingState::Member
    );
}

#[test]
fn retained_standing_can_expire_for_the_same_lifecycle_epoch() {
    let store = Store::open_memory().unwrap();
    store.grant_session_route("pk", "room", 1).unwrap();
    store
        .conn
        .execute(
            "INSERT INTO session_standing VALUES ('pk','room','retained',100,1,7,10)",
            [],
        )
        .unwrap();
    let epoch = 1;
    assert!(
        store
            .mark_session_standing_absent_if_epoch(
                "pk",
                "room",
                StandingState::Retained,
                epoch,
                7,
                100,
            )
            .unwrap()
    );
    let row = store.get_session_standing("pk", "room").unwrap().unwrap();
    assert_eq!(row.state, StandingState::Absent);
    assert_eq!(row.retain_until, 0);
    assert_eq!(row.standing_epoch, epoch + 1);
}
