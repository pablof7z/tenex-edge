#[test]
fn retryable_outbox_failed_publish_stays_pending() {
    let store = crate::state::Store::open_memory().unwrap();
    let id = store.enqueue_outbox("{\"kind\":30315}", 1).unwrap();

    store
        .apply_outbox_projection(id, "pending", Some("relay timeout"), true)
        .unwrap();

    let pending = store.peek_outbox(10, u64::MAX).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].local_id, id);
    assert_eq!(pending[0].retries, 1);
    assert_eq!(pending[0].last_error.as_deref(), Some("relay timeout"));
}

#[test]
fn terminal_outbox_failed_publish_leaves_pending_queue() {
    let store = crate::state::Store::open_memory().unwrap();
    let id = store.enqueue_outbox("{\"kind\":30315}", 1).unwrap();

    store
        .apply_outbox_projection(
            id,
            "failed",
            Some("relay rejected event: blocked: unknown member"),
            true,
        )
        .unwrap();

    assert!(store.peek_outbox(10, u64::MAX).unwrap().is_empty());
    let row = store.get_outbox(id).unwrap().unwrap();
    assert_eq!(row.state, "failed");
    assert_eq!(row.retries, 1);
    assert_eq!(
        row.last_error.as_deref(),
        Some("relay rejected event: blocked: unknown member")
    );
}
