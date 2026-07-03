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

#[test]
fn archived_channel_predicate_uses_about_prefix() {
    let store = Store::open_memory().unwrap();
    store
        .upsert_channel("active", "Active", "normal work", "project", 1)
        .unwrap();
    store
        .upsert_channel("archived", "Archived", "[ARCHIVED] done", "project", 1)
        .unwrap();

    assert!(is_archived_channel_about("[ARCHIVED] done"));
    assert!(!is_archived_channel_about("done [ARCHIVED]"));
    assert_eq!(archived_channel_about(""), "[ARCHIVED]");
    assert_eq!(archived_channel_about("done"), "[ARCHIVED] done");
    assert_eq!(archived_channel_about("[ARCHIVED] done"), "[ARCHIVED] done");
    assert_eq!(
        archived_channel_about(&"a".repeat(CHANNEL_ABOUT_MAX_CHARS))
            .chars()
            .count(),
        CHANNEL_ABOUT_MAX_CHARS
    );
    assert!(!store.is_archived_channel("active").unwrap());
    assert!(store.is_archived_channel("archived").unwrap());
    assert!(!store.is_archived_channel("missing").unwrap());
}
