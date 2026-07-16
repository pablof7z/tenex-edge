use super::*;
use crate::instrument::changed_summary_json;
use crate::state::receipts::NewReceipt;

#[test]
fn event_resolves_matching_receipts() {
    let store = Store::open_memory().unwrap();
    store
        .record_receipt(&NewReceipt {
            surface: "status".into(),
            transaction_id: 5,
            revision: 2,
            changed_summary: changed_summary_json(&[], &[], &[], Some("sid-1")),
            commands: "[]".into(),
            artifact_ref: Some("event-1".into()),
            created_at: 10,
        })
        .unwrap();

    let value = explain(&store, &Handle::Event("event-1".into())).unwrap();
    assert_eq!(value["kind"], "event");
    assert_eq!(value["receipts"][0]["artifact_ref"], "event-1");
}

#[test]
fn session_resolves_latest_matching_status_receipt() {
    let store = Store::open_memory().unwrap();
    for (pubkey, at) in [("sid-1", 10), ("sid-2", 20), ("sid-1", 30)] {
        store
            .record_receipt(&NewReceipt {
                surface: "status".into(),
                transaction_id: at,
                revision: 1,
                changed_summary: changed_summary_json(&[], &[], &[], Some(pubkey)),
                commands: "[]".into(),
                artifact_ref: Some(format!("event-{at}")),
                created_at: at,
            })
            .unwrap();
    }

    let value = explain(
        &store,
        &Handle::Session {
            id: "sid-1".into(),
            at: None,
        },
    )
    .unwrap();
    assert_eq!(value["receipts"][0]["artifact_ref"], "event-30");
}

#[test]
fn parse_rejects_removed_llm_handle() {
    let error = parse_handle("llm:1").unwrap_err();
    assert!(error.to_string().contains("unknown handle scheme"));
}
