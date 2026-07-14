use super::*;

#[tokio::test]
async fn rpc_probe_validate_member_passes_for_member_row() {
    let state = DaemonState::new_for_test().await;
    seed_roster(&state);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "member:room:pk-member" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "membership", "passed");
    assert_eq!(v["membership_evidence"]["role"], "member");
    assert_eq!(v["membership_evidence"]["membership_snapshot"], true);
}

#[tokio::test]
async fn rpc_probe_validate_admin_passes_for_admin_row() {
    let state = DaemonState::new_for_test().await;
    seed_roster(&state);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "admin/room/pk-admin" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "membership", "passed");
    assert_eq!(v["membership_evidence"]["role"], "admin");
    assert_eq!(v["membership_evidence"]["require_admin"], true);
}

#[tokio::test]
async fn rpc_probe_validate_admin_fails_for_plain_member() {
    let state = DaemonState::new_for_test().await;
    seed_roster(&state);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "admin:room:pk-member" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "membership", "failed");
    assert_eq!(v["membership_evidence"]["role"], "member");
}

#[tokio::test]
async fn rpc_probe_validate_member_absence_fails_when_snapshot_hydrated() {
    let state = DaemonState::new_for_test().await;
    seed_roster(&state);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "membership:room:pk-missing" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "membership", "failed");
    assert_eq!(v["membership_evidence"]["found"], false);
    assert_eq!(v["membership_evidence"]["membership_snapshot"], true);
}

#[tokio::test]
async fn rpc_probe_validamosaico_optimistic_member_passes_with_limitation_before_snapshot() {
    let state = DaemonState::new_for_test().await;
    state
        .with_store(|s| {
            s.upsert_channel("room", "Room", "", "", 100)?;
            s.upsert_channel_member("room", "pk-member", "member", 101)?;
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "member:room:pk-member" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "membership", "passed");
    assert_eq!(v["membership_evidence"]["membership_snapshot"], false);
    assert!(v["limitations"].as_array().unwrap().iter().any(|l| {
        l.as_str()
            .unwrap()
            .contains("complete admin/member snapshots are not hydrated")
    }));
}

#[tokio::test]
async fn rpc_probe_validate_membership_snapshot_passes_when_both_sets_hydrated() {
    let state = DaemonState::new_for_test().await;
    seed_roster(&state);

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "membership_snapshot:room" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "membership_snapshot", "passed");
    assert_eq!(v["membership_snapshot_evidence"]["snapshot_complete"], true);
    assert_eq!(v["membership_snapshot_evidence"]["admin_count"], 1);
    assert_eq!(v["membership_snapshot_evidence"]["member_count"], 2);
    assert_eq!(v["membership_snapshot_evidence"]["admin_set_found"], true);
    assert_eq!(v["membership_snapshot_evidence"]["member_set_found"], true);
}

#[tokio::test]
async fn rpc_probe_validate_membership_snapshot_reports_incomplete_as_not_proven() {
    let state = DaemonState::new_for_test().await;
    state
        .with_store(|s| {
            s.upsert_channel("room", "Room", "", "", 100)?;
            s.replace_channel_members("room", &["pk-member".to_string()], 102)?;
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "roster:room" }),
    )
    .unwrap();

    assert_eq!(v["ok"], true);
    assert_check_status(&v, "membership_snapshot", "not_proven");
    assert_eq!(
        v["membership_snapshot_evidence"]["snapshot_complete"],
        false
    );
    assert_eq!(v["membership_snapshot_evidence"]["admin_set_found"], false);
    assert_eq!(v["membership_snapshot_evidence"]["member_set_found"], true);
}

#[tokio::test]
async fn rpc_probe_validate_membership_snapshot_fails_when_snapshot_has_no_admin() {
    let state = DaemonState::new_for_test().await;
    state
        .with_store(|s| {
            s.upsert_channel("room", "Room", "", "", 100)?;
            s.replace_channel_admins("room", &[], 101)?;
            s.replace_channel_members("room", &["pk-member".to_string()], 102)?;
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();

    let v = rpc_probe(
        &state,
        &json!({ "verb": "validate", "target": "membership_snapshot:room" }),
    )
    .unwrap();

    assert_eq!(v["ok"], false);
    assert_check_status(&v, "membership_snapshot", "failed");
    assert_eq!(v["membership_snapshot_evidence"]["snapshot_complete"], true);
    assert_eq!(v["membership_snapshot_evidence"]["admin_count"], 0);
}

fn seed_roster(state: &std::sync::Arc<DaemonState>) {
    state
        .with_store(|s| {
            s.upsert_channel("room", "Room", "", "", 100)?;
            s.replace_channel_admins("room", &["pk-admin".to_string()], 101)?;
            s.replace_channel_members("room", &["pk-member".to_string()], 102)?;
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();
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
