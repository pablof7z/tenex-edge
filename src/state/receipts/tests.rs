use crate::state::{receipts::NewReceipt, Store};

fn receipt(surface: &str, artifact_ref: Option<&str>, created_at: i64) -> NewReceipt {
    NewReceipt {
        surface: surface.into(),
        transaction_id: 42,
        revision: 7,
        changed_summary: r#"{"added":1,"removed":0}"#.into(),
        commands: r#"[{"kind":"publish","key":"k1","reason":"changed"}]"#.into(),
        artifact_ref: artifact_ref.map(str::to_string),
        created_at,
    }
}

#[test]
fn record_then_get_round_trips() {
    let s = Store::open_memory().unwrap();
    let id = s
        .record_receipt(&receipt("status", Some("evt-1"), 1_000))
        .unwrap();

    let row = s.get_receipt(id).unwrap().unwrap();
    assert_eq!(row.id, id);
    assert_eq!(row.surface, "status");
    assert_eq!(row.transaction_id, 42);
    assert_eq!(row.revision, 7);
    assert_eq!(row.artifact_ref.as_deref(), Some("evt-1"));
    assert_eq!(row.created_at, 1_000);
}

#[test]
fn get_missing_id_returns_none() {
    let s = Store::open_memory().unwrap();
    assert!(s.get_receipt(999).unwrap().is_none());
}

#[test]
fn latest_for_surface_orders_newest_first_and_respects_limit() {
    let s = Store::open_memory().unwrap();
    s.record_receipt(&receipt("status", Some("evt-1"), 1_000))
        .unwrap();
    s.record_receipt(&receipt("status", Some("evt-2"), 3_000))
        .unwrap();
    s.record_receipt(&receipt("status", Some("evt-3"), 2_000))
        .unwrap();
    // Different surface must not leak in.
    s.record_receipt(&receipt("hook_context", Some("evt-4"), 4_000))
        .unwrap();

    let rows = s.latest_receipts_for_surface("status", 2).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].created_at, 3_000);
    assert_eq!(rows[1].created_at, 2_000);
}

#[test]
fn by_artifact_ref_filters_and_orders_oldest_first() {
    let s = Store::open_memory().unwrap();
    s.record_receipt(&receipt("status", Some("evt-a"), 2_000))
        .unwrap();
    s.record_receipt(&receipt("hook_context", Some("evt-a"), 1_000))
        .unwrap();
    s.record_receipt(&receipt("subscriptions", Some("evt-b"), 500))
        .unwrap();

    let rows = s.receipts_by_artifact_ref("evt-a").unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].created_at, 1_000);
    assert_eq!(rows[1].created_at, 2_000);
}

#[test]
fn by_artifact_ref_prefix_requires_one_artifact() {
    let s = Store::open_memory().unwrap();
    s.record_receipt(&receipt("status", Some("evt-abc"), 2_000))
        .unwrap();
    s.record_receipt(&receipt("hook_context", Some("evt-abc"), 1_000))
        .unwrap();
    s.record_receipt(&receipt("status", Some("evt-def"), 3_000))
        .unwrap();

    let rows = s.receipts_by_artifact_ref_prefix("evt-a").unwrap();
    assert_eq!(rows.len(), 2);
    assert!(s.receipts_by_artifact_ref_prefix("evt-").is_err());
}

#[test]
fn find_near_picks_closest_created_at() {
    let s = Store::open_memory().unwrap();
    s.record_receipt(&receipt("hook_context", Some("evt-1"), 1_000))
        .unwrap();
    s.record_receipt(&receipt("hook_context", Some("evt-2"), 5_000))
        .unwrap();
    s.record_receipt(&receipt("hook_context", Some("evt-3"), 9_000))
        .unwrap();

    let row = s.find_receipt_near("hook_context", 6_000).unwrap().unwrap();
    assert_eq!(row.artifact_ref.as_deref(), Some("evt-2"));

    // No rows for an unknown surface.
    assert!(s
        .find_receipt_near("subscriptions", 6_000)
        .unwrap()
        .is_none());
}
