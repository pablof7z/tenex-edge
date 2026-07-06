use super::*;
use crate::state::NewChannelReadinessAttempt;

#[tokio::test]
async fn rpc_probe_validate_readiness_attempt_passes_when_current_state_corrobates_ready() {
    let state = DaemonState::new_for_test().await;
    let id = state
        .with_store(|s| {
            s.upsert_channel("room", "Room", "work room", "", 100)?;
            s.replace_channel_admins("room", &["pk-admin".to_string()], 101)?;
            s.replace_channel_members("room", &["pk-member".to_string()], 102)?;
            s.record_channel_readiness_attempt(&NewChannelReadinessAttempt {
                channel_h: "room".into(),
                expect_member: "pk-member".into(),
                parent_hint: Some("root".into()),
                name: Some("Room".into()),
                source: "provider.ensure_channel_ready".into(),
                outcome: "ready".into(),
                reason: "channel readiness verified".into(),
                created_at: 103,
            })
        })
        .unwrap();

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("readiness_attempt:{id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert!(v["target_evidence"].is_null());
    assert_check_status(&v, "readiness_attempt", "passed");
    assert_eq!(v["readiness_attempt_evidence"]["found"], true);
    assert_eq!(v["readiness_attempt_evidence"]["id"], id);
    assert_eq!(v["readiness_attempt_evidence"]["current_ready"], true);
    assert_eq!(
        v["readiness_attempt_evidence"]["expected_member_role"],
        "member"
    );
}

#[tokio::test]
async fn rpc_probe_validate_readiness_attempt_fails_for_degraded_outcome() {
    let state = DaemonState::new_for_test().await;
    let id = state
        .with_store(|s| {
            s.record_channel_readiness_attempt(&NewChannelReadinessAttempt {
                channel_h: "missing".into(),
                expect_member: "pk-member".into(),
                parent_hint: Some("root".into()),
                name: Some("Missing".into()),
                source: "provider.ensure_channel_ready".into(),
                outcome: "degraded".into(),
                reason: "management key is not admin and self-grant failed".into(),
                created_at: 100,
            })
        })
        .unwrap();

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": format!("provider_attempt:{id}") }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "readiness_attempt", "failed");
    assert_eq!(v["readiness_attempt_evidence"]["degraded_outcome"], true);
    assert!(v["readiness_attempt_evidence"]["reason"]
        .as_str()
        .unwrap()
        .contains("self-grant failed"));
}

#[tokio::test]
async fn rpc_probe_validate_readiness_attempt_reports_missing_as_not_proven() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "readiness_attempt:99" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "readiness_attempt", "not_proven");
    assert_eq!(v["readiness_attempt_evidence"]["found"], false);
    assert!(v["readiness_attempt_evidence"]["reason"]
        .as_str()
        .unwrap()
        .contains("no channel_readiness_attempts row"));
}

#[tokio::test]
async fn rpc_probe_validate_readiness_attempt_reports_invalid_id() {
    let state = DaemonState::new_for_test().await;

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "readiness_attempt:not-a-number" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "target", "failed");
    assert_eq!(v["target_evidence"]["kind"], "invalid_probe_handle");
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
