use super::*;

pub(super) fn advertise_host(
    store: &Store,
    pubkey: &str,
    host: &str,
    agents: &[(&str, &str)],
    workspaces: &[&str],
    updated_at: u64,
) {
    let agents = agents
        .iter()
        .map(|(slug, about)| ((*slug).to_string(), (*about).to_string()))
        .collect::<Vec<_>>();
    let workspaces = workspaces
        .iter()
        .map(|workspace| (*workspace).to_string())
        .collect::<Vec<_>>();
    store
        .upsert_profile_snapshot(
            pubkey,
            host,
            host,
            "",
            host,
            true,
            &agents,
            &workspaces,
            updated_at,
        )
        .unwrap();
    for workspace in workspaces {
        store
            .upsert_channel_member(&workspace, pubkey, "admin", updated_at)
            .unwrap();
    }
}

#[test]
fn agent_context_omits_capabilities_while_human_view_preserves_them() {
    let store = seed_store();
    store
        .upsert_channel("other", "other", "Other workspace", "", 1)
        .unwrap();
    advertise_host(
        &store,
        "backend",
        "laptop",
        &[
            ("shared", "Available everywhere"),
            ("other-only", "Only in other"),
        ],
        &["root", "other"],
        2,
    );

    let roots = vec!["root".into(), "other".into()];
    let rendered = render_fabric_all_workspaces(&store, &roots, 100, "laptop", "");
    assert_eq!(rendered.matches("<mosaico>").count(), 1, "got: {rendered}");
    assert!(!rendered.contains("mosaico agents list"), "got: {rendered}");
    assert!(!rendered.contains("<available-agents>"), "got: {rendered}");
    assert!(!rendered.contains("<workspace-agents>"), "got: {rendered}");
    assert!(!rendered.contains("@shared"), "got: {rendered}");
    assert!(!rendered.contains("@other-only"), "got: {rendered}");

    let human =
        render_fabric_all_workspaces_human(&store, &roots, 100, "laptop", "", false).unwrap();
    assert_eq!(
        human.matches("Available agents (all workspaces)").count(),
        1,
        "got: {human}"
    );
    assert_eq!(human.matches("@shared").count(), 1, "got: {human}");
    assert_eq!(human.matches("@other-only").count(), 1, "got: {human}");
    assert!(!human.contains("Workspace-specific agents"), "got: {human}");
}
