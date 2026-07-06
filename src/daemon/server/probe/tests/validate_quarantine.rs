use super::*;
use crate::state::RelayEvent;

#[tokio::test]
async fn rpc_probe_validate_quarantine_reports_blocked_event() {
    let state = DaemonState::new_for_test().await;
    seed_quarantine(&state, "evt-q-123", "room roster is not hydrated");

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "quarantine:evt-q" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "quarantine", "failed");
    assert_eq!(v["quarantine_evidence"]["found"], true);
    assert_eq!(v["quarantine_evidence"]["row_count"], 1);
    assert_eq!(
        v["quarantine_evidence"]["rows"][0]["reason"],
        "room roster is not hydrated"
    );
}

#[tokio::test]
async fn rpc_probe_validate_event_reports_quarantine_failure() {
    let state = DaemonState::new_for_test().await;
    seed_quarantine(&state, "evt-q-456", "author is not an admitted member");

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "event:evt-q-456" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "event", "failed");
    assert_eq!(v["event_evidence"]["quarantine_found"], true);
    assert_eq!(v["event_evidence"]["quarantine_count"], 1);
    assert!(v["limitations"].as_array().unwrap().iter().any(|l| {
        l.as_str()
            .unwrap()
            .contains("quarantined and has not been admitted")
    }));
}

#[tokio::test]
async fn rpc_probe_validate_quarantine_passes_materialized_unquarantined_event() {
    let state = DaemonState::new_for_test().await;
    state
        .with_store(|s| {
            s.insert_event(&relay_event("evt-ok-123"))?;
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "quarantine:evt-ok" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "quarantine", "passed");
    assert_eq!(v["quarantine_evidence"]["found"], false);
    assert_eq!(v["quarantine_evidence"]["relay_event_found"], true);
}

fn seed_quarantine(state: &std::sync::Arc<DaemonState>, id: &str, reason: &str) {
    state
        .with_store(|s| {
            let ev = relay_event(id);
            let event_json =
                format!(r#"{{"id":"{id}","kind":9,"pubkey":"pk-author","content":"hello"}}"#);
            s.quarantine_event(&ev, &event_json, reason, 120)?;
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();
}

fn relay_event(id: &str) -> RelayEvent {
    RelayEvent {
        id: id.into(),
        kind: 9,
        pubkey: "pk-author".into(),
        created_at: 100,
        channel_h: "room".into(),
        d_tag: String::new(),
        content: "hello".into(),
        tags_json: "[]".into(),
    }
}

fn assert_check_status(v: &serde_json::Value, name: &str, status: &str) {
    assert_eq!(check_row(v, name)["status"], status);
}

fn check_row<'a>(v: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    v["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .expect("check row")
}
