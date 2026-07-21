use super::*;

#[test]
fn discovers_agent_like_named_codex_profiles_from_codex_home() {
    let home = TempDir::new().unwrap();
    let profile = home
        .path()
        .join(".codex/visual-language-partner.config.toml");
    write(
        &profile,
        r#"# Summary: Develops a durable visual language.
model_reasoning_effort = "high"
developer_instructions = "Create original visual narratives"
"#,
    );
    write(
        &home.path().join(".codex/ci.config.toml"),
        "model_reasoning_effort='high'",
    );
    write(
        &home.path().join(".codex/config.toml"),
        "developer_instructions='base instructions are not a named profile'",
    );

    let catalog = AgentCatalog::discover(&roots(home.path()), &[]).unwrap();
    assert_eq!(catalog.slugs(), ["visual-language-partner"]);

    let discovered = catalog
        .resolve("visual-language-partner", None, None)
        .unwrap();
    assert_eq!(discovered.path, profile);
    assert_eq!(
        discovered.use_criteria,
        "Develops a durable visual language."
    );
    let NativeAgentActivation::CodexRoot(activation) = discovered.activation().unwrap() else {
        panic!("expected Codex root activation");
    };
    assert_eq!(
        activation.developer_instructions,
        "Create original visual narratives"
    );
    assert_eq!(
        activation.config["model_reasoning_effort"].as_str(),
        Some("high")
    );
}

#[test]
fn named_codex_profile_wins_same_slug_custom_agent_for_root_launch() {
    let home = TempDir::new().unwrap();
    write(
        &home.path().join(".codex/agents/reviewer.toml"),
        "name='reviewer'\ndescription='Spawned reviewer'\ndeveloper_instructions='Spawn'",
    );
    let named = home.path().join(".codex/reviewer.config.toml");
    write(
        &named,
        "developer_instructions='Root reviewer'\nmodel_reasoning_effort='high'",
    );

    let catalog = AgentCatalog::discover(&roots(home.path()), &[]).unwrap();
    let discovered = catalog.resolve("reviewer", None, None).unwrap();
    assert_eq!(discovered.path, named);
    let NativeAgentActivation::CodexRoot(activation) = discovered.activation().unwrap() else {
        panic!("expected Codex root activation");
    };
    assert_eq!(activation.developer_instructions, "Root reviewer");
}

#[test]
fn installed_roots_honor_codex_home_for_agents_and_named_profiles() {
    let home = TempDir::new().unwrap();
    let codex_home = home.path().join("custom-codex");
    let mut env = crate::test_env::EnvGuard::set("HOME", home.path());
    env.set_var("CODEX_HOME", &codex_home);

    let roots = DiscoveryRoots::installed().unwrap();
    assert_eq!(roots.codex_profiles, codex_home);
    assert_eq!(roots.codex, codex_home.join("agents"));
}
