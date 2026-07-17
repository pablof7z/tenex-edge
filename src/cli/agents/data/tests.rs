use super::*;
use crate::agent_catalog::DiscoveryRoots;

fn write(path: &Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

#[test]
fn combines_configured_native_and_generic_agents() {
    let home = tempfile::tempdir().unwrap();
    write(
        &home.path().join(".claude/agents/reviewer.md"),
        "---\nname: reviewer\ndescription: Reviews changes\n---\nReview",
    );
    let catalog = AgentCatalog::discover(&DiscoveryRoots::for_user_home(home.path()), &[]).unwrap();
    let harnesses: HarnessesConfig = serde_json::from_str(
        r#"{"claude-pty":{"harness":"claude","transport":"pty"},"codex-pty":{"harness":"codex","transport":"pty"}}"#,
    )
    .unwrap();
    crate::identity::add_local_agent(home.path(), "writer", "codex-pty", None, 1).unwrap();
    let rows = build(
        home.path(),
        &[Harness::ClaudeCode, Harness::Codex],
        &harnesses,
        &catalog,
        home.path(),
    );
    assert!(rows.iter().any(|row| row.slug == "reviewer"));
    assert!(rows.iter().any(|row| row.slug == "writer"));
    assert_eq!(
        rows.iter()
            .find(|row| row.slug == "codex")
            .unwrap()
            .description,
        "Generic Codex agent"
    );
}

#[test]
fn configured_native_profile_is_one_row_with_exact_profile_attached() {
    let home = tempfile::tempdir().unwrap();
    write(
        &home.path().join(".claude/agents/reviewer.md"),
        "---\nname: reviewer\ndescription: Reviews changes\n---\nReview",
    );
    let catalog = AgentCatalog::discover(&DiscoveryRoots::for_user_home(home.path()), &[]).unwrap();
    let harnesses: HarnessesConfig =
        serde_json::from_str(r#"{"claude-pty":{"harness":"claude","transport":"pty"}}"#).unwrap();
    crate::identity::add_local_agent(home.path(), "reviewer", "claude-pty", None, 1).unwrap();
    let rows = build(
        home.path(),
        &[Harness::ClaudeCode],
        &harnesses,
        &catalog,
        home.path(),
    );
    let reviewer = rows.iter().find(|row| row.slug == "reviewer").unwrap();
    assert_eq!(reviewer.kind, AgentKind::Configured);
    assert!(reviewer.native_profile.is_some());
}

#[test]
fn native_profile_preview_never_selects_an_unsupported_transport() {
    let harnesses: HarnessesConfig = serde_json::from_str(
        r#"{
          "opencode-acp":{"harness":"opencode","transport":"acp"},
          "opencode-pty":{"harness":"opencode","transport":"pty"}
        }"#,
    )
    .unwrap();

    assert_eq!(
        preferred_bundle(&harnesses, Harness::Opencode, true).as_deref(),
        Some("opencode-pty")
    );
    assert_eq!(
        preferred_bundle(&harnesses, Harness::Opencode, false).as_deref(),
        Some("opencode-acp")
    );
}
