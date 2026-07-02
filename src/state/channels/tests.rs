use super::*;

#[test]
fn projects_read_model_returns_root_channels_sorted_by_id() {
    let store = Store::open_memory().unwrap();
    store
        .upsert_channel("task", "Task", "not a project", "beta", 3)
        .unwrap();
    store.upsert_channel("beta", "Beta", "two", "", 2).unwrap();
    store
        .upsert_channel("alpha", "Alpha", "one", "", 1)
        .unwrap();

    let projects = store.list_projects_read_model().unwrap();
    let ids = projects
        .iter()
        .map(|channel| channel.channel_h.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["alpha", "beta"]);
    assert_eq!(projects[0].human_name(), Some("Alpha"));
    assert_eq!(projects[0].about, "one");
}

#[test]
fn channel_meta_read_model_returns_parent_metadata() {
    let store = Store::open_memory().unwrap();
    store
        .upsert_channel("task", "Task", "child", "project", 1)
        .unwrap();

    let meta = store.channel_meta_read_model("task").unwrap().unwrap();
    assert_eq!(meta.parent, "project");
    assert_eq!(meta.human_name(), Some("Task"));
}
