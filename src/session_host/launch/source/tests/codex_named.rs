use super::*;

#[tokio::test]
async fn installed_named_codex_profile_resolves_without_agent_json() {
    let home = tempfile::tempdir().unwrap();
    let mosaico_home = home.path().join("mosaico");
    let codex_home = home.path().join(".codex");
    let mut env = EnvGuard::set("MOSAICO_HOME", &mosaico_home);
    env.set_var("MOSAICO_ISOLATED_HOME_OK", "1");
    env.set_var("HOME", home.path());
    env.set_var("CODEX_HOME", &codex_home);
    write(
        &mosaico_home.join("harnesses.json"),
        r#"{"codex-rpc":{"harness":"codex","transport":"app-server"}}"#,
    );
    write(
        &codex_home.join("visual-language-partner.config.toml"),
        "developer_instructions='Create visual narratives'\nmodel_reasoning_effort='high'",
    );
    write_executable(&home.path().join(".local/bin/codex"));
    let workspace = home.path().join("work");
    std::fs::create_dir_all(&workspace).unwrap();
    let state = DaemonState::new_for_test().await;
    state.refresh_agent_catalog().unwrap();

    let source = resolve_agent_source(
        &state,
        "visual-language-partner",
        &workspace,
        LaunchIntent::Managed,
    )
    .unwrap();

    assert_eq!(source.bundle, "codex-rpc");
    let Some(NativeAgentActivation::CodexRoot(activation)) = source.native_agent else {
        panic!("expected named profile root activation");
    };
    assert_eq!(
        activation.developer_instructions,
        "Create visual narratives"
    );
    assert_eq!(
        activation.config["model_reasoning_effort"].as_str(),
        Some("high")
    );
    assert!(!mosaico_home
        .join("agents/visual-language-partner.json")
        .exists());
}

#[tokio::test]
async fn configured_claude_binding_can_rebind_to_discovered_codex_profile() {
    let home = tempfile::tempdir().unwrap();
    let mosaico_home = home.path().join("mosaico");
    let codex_home = home.path().join(".codex");
    let mut env = EnvGuard::set("MOSAICO_HOME", &mosaico_home);
    env.set_var("MOSAICO_ISOLATED_HOME_OK", "1");
    env.set_var("HOME", home.path());
    env.set_var("CODEX_HOME", &codex_home);
    write(
        &mosaico_home.join("harnesses.json"),
        r#"{
          "claude-pty":{"harness":"claude-code","transport":"pty"},
          "codex-pty":{"harness":"codex","transport":"pty"}
        }"#,
    );
    write(
        &home
            .path()
            .join(".claude/agents/visual-language-partner.md"),
        "---\nname: visual-language-partner\ndescription: Creates visuals\n---\nCreate",
    );
    write(
        &codex_home.join("visual-language-partner.config.toml"),
        "developer_instructions='Create visual narratives'",
    );
    for executable in ["claude", "codex"] {
        write_executable(&home.path().join(".local/bin").join(executable));
    }
    crate::identity::add_local_agent(
        &mosaico_home,
        "visual-language-partner",
        "claude-pty",
        None,
        10,
    )
    .unwrap();
    let workspace = home.path().join("work");
    std::fs::create_dir_all(&workspace).unwrap();
    let state = DaemonState::new_for_test().await;
    state.refresh_agent_catalog().unwrap();

    let source = resolve_agent_source(
        &state,
        "visual-language-partner-codex",
        &workspace,
        LaunchIntent::Interactive,
    )
    .unwrap();

    assert_eq!(source.bundle, "codex-pty");
    assert_eq!(source.identity.slug, "visual-language-partner");
    assert!(matches!(
        source.native_agent,
        Some(NativeAgentActivation::CodexRoot(_))
    ));
    let saved =
        crate::identity::agent_launch_config(&mosaico_home, "visual-language-partner").unwrap();
    assert_eq!(saved.harness, "codex-pty");
    assert!(saved.profile.is_none());
}
