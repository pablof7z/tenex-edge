use super::NewCommit;
use crate::state::Store;

fn commit(surface: &str, noop: i64, commands: i64, created_at: i64) -> NewCommit {
    NewCommit {
        surface: surface.into(),
        transaction_id: 42,
        revision: 7,
        mode: "authoritative".into(),
        trigger_kind: "tick".into(),
        trigger_ref: "s1".into(),
        changed_inputs_json: r#"["status/s1/activity"]"#.into(),
        changed_derived_json: r#"["status/s1/content"]"#.into(),
        changed_collections_json: "[]".into(),
        resource_commands_json: "[]".into(),
        output_frames_json: "[]".into(),
        command_count: commands,
        output_count: 0,
        effect_count: commands,
        suppressed_count: noop,
        noop,
        oracle_status: None,
        oracle_error: None,
        duration_us: 250,
        graph_nodes: 6,
        graph_resources: 2,
        created_at,
    }
}

#[test]
fn record_then_latest_orders_newest_first_and_filters_surface() {
    let s = Store::open_memory().unwrap();
    s.record_commit(&commit("status", 0, 1, 1_000)).unwrap();
    s.record_commit(&commit("status", 1, 0, 3_000)).unwrap();
    s.record_commit(&commit("status", 0, 2, 2_000)).unwrap();
    s.record_commit(&commit("subscriptions", 0, 1, 4_000))
        .unwrap();

    let rows = s.latest_commits_for_surface("status", 10).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].created_at, 3_000);
    assert_eq!(rows[0].noop, 1);
    assert_eq!(rows[0].mode, "authoritative");
    assert_eq!(rows[0].trigger_ref, "s1");
    assert_eq!(rows[0].suppressed_count, 1);
    assert_eq!(rows[0].graph_resources, 2);
    assert_eq!(rows[2].created_at, 1_000);
    assert_eq!(rows[0].changed_inputs_json, r#"["status/s1/activity"]"#);
}

#[test]
fn stats_aggregate_effectful_and_noop() {
    let s = Store::open_memory().unwrap();
    s.record_commit(&commit("status", 0, 1, 1_000)).unwrap();
    s.record_commit(&commit("status", 1, 0, 2_000)).unwrap();
    s.record_commit(&commit("status", 0, 2, 3_000)).unwrap();
    s.record_commit(&commit("status", 0, 5, 500)).unwrap();

    let stats = s.commit_stats("status", 1_000).unwrap();
    assert_eq!(stats.commits, 3);
    assert_eq!(stats.effectful, 2);
    assert_eq!(stats.noop, 1);
    assert_eq!(stats.command_count_sum, 3);
    assert_eq!(stats.effect_count_sum, 3);
    assert_eq!(stats.suppressed_count_sum, 1);
    assert_eq!(stats.max_graph_nodes, 6);
    assert_eq!(stats.max_graph_resources, 2);
    assert_eq!(stats.latest_graph_resources, 2);
    assert_eq!(stats.open_count, 0);
    assert_eq!(stats.duration_us_sum, 750);
}

#[test]
fn subscription_live_balance_resets_on_transaction_epoch_restart() {
    let s = Store::open_memory().unwrap();

    let mut old_epoch = commit("subscriptions", 0, 2, 1_000);
    old_epoch.transaction_id = 20;
    old_epoch.revision = 2;
    old_epoch.resource_commands_json = r#"[{"kind":"open"},{"kind":"open"}]"#.into();
    old_epoch.graph_resources = 2;
    s.record_commit(&old_epoch).unwrap();

    let mut new_epoch = commit("subscriptions", 0, 3, 2_000);
    new_epoch.transaction_id = 1;
    new_epoch.revision = 2;
    new_epoch.resource_commands_json =
        r#"[{"kind":"open"},{"kind":"open"},{"kind":"open"}]"#.into();
    new_epoch.graph_resources = 3;
    s.record_commit(&new_epoch).unwrap();

    let stats = s.commit_stats("subscriptions", 0).unwrap();
    assert_eq!(stats.open_count, 5);
    assert_eq!(stats.live_resource_balance, 3);
    assert_eq!(stats.latest_graph_resources, 3);
    assert!(!stats.resource_drift);
}

#[test]
fn stats_over_empty_surface_is_zeroed() {
    let s = Store::open_memory().unwrap();
    let stats = s.commit_stats("hook_context", 0).unwrap();
    assert_eq!(stats.commits, 0);
    assert_eq!(stats.effectful, 0);
    assert_eq!(stats.max_graph_nodes, 0);
}

#[test]
fn oracle_sample_stamps_latest_surface_commit() {
    let s = Store::open_memory().unwrap();
    s.record_commit(&commit("status", 0, 1, 1_000)).unwrap();
    s.record_commit(&commit("status", 0, 1, 2_000)).unwrap();

    assert_eq!(s.record_oracle_sample("status", "green", None).unwrap(), 1);
    let rows = s.latest_commits_for_surface("status", 2).unwrap();
    assert_eq!(rows[0].oracle_status.as_deref(), Some("green"));
    assert_eq!(rows[1].oracle_status, None);
}
