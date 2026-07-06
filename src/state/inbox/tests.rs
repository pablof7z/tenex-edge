use super::*;

fn state_for(s: &Store, event_id: &str, target_key: &str) -> String {
    s.conn
        .query_row(
            "SELECT state FROM inbox WHERE event_id=?1 AND target_session=?2",
            params![event_id, target_key],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn orchestration_target_claim_retries_only_failed_targets() {
    let s = Store::open_memory().unwrap();

    assert!(s
        .claim_orchestration_target("ev", "orchestration:backend:0:a", "admin", "child", "a", 10,)
        .unwrap());
    assert!(s
        .claim_orchestration_target("ev", "orchestration:backend:1:b", "admin", "child", "b", 10,)
        .unwrap());

    s.retry_orchestration_target("ev", "orchestration:backend:0:a")
        .unwrap();
    s.complete_orchestration_target("ev", "orchestration:backend:1:b", 11)
        .unwrap();

    assert!(
        s.claim_orchestration_target("ev", "orchestration:backend:0:a", "admin", "child", "a", 12,)
            .unwrap(),
        "failed target should be retryable"
    );
    assert!(
        !s.claim_orchestration_target(
            "ev",
            "orchestration:backend:1:b",
            "admin",
            "child",
            "b",
            12,
        )
        .unwrap(),
        "completed target should not be reprocessed"
    );
    assert_eq!(
        state_for(&s, "ev", "orchestration:backend:0:a"),
        "processing"
    );
    assert_eq!(
        state_for(&s, "ev", "orchestration:backend:1:b"),
        "delivered"
    );
}

#[test]
fn inbox_event_prefix_lookup_can_filter_target() {
    let s = Store::open_memory().unwrap();
    s.enqueue_inbox("evt-abc", "s1", "pk", "room", "one", 10)
        .unwrap();
    s.enqueue_inbox("evt-abc", "s2", "pk", "room", "two", 11)
        .unwrap();
    s.enqueue_inbox("evt-other", "s1", "pk", "room", "three", 12)
        .unwrap();

    let rows = s.inbox_by_event_prefix("evt-a").unwrap();
    assert_eq!(rows.len(), 2);

    let row = s.inbox_by_event_prefix_and_target("evt-a", "s2").unwrap();
    assert_eq!(row.len(), 1);
    assert_eq!(row[0].body, "two");
}
