use super::*;

#[test]
fn missing_provider_is_a_cli_failure_without_live_metadata_or_session() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let home = Home::new();
    write_config(&home, false);
    let channel = unique_session("missing-provider");
    let work_dir = home.dir.path().join(&channel);
    add_workspace_mapping(&home, &channel, &work_dir);
    std::fs::write(
        home.dir.path().join("harnesses.json"),
        r#"{"missing-grok":{"harness":"grok","transport":"pty"}}"#,
    )
    .unwrap();
    mosaico::identity::add_local_agent(
        home.dir.path(),
        "missing-provider-role",
        "missing-grok",
        None,
        1,
    )
    .unwrap();

    let output = run_cli_with_env_in_dir(
        &home,
        &["agents", "missing-provider-role", "--workspace", &channel],
        &[("PATH", "/usr/bin:/bin")],
        &work_dir,
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("launch of agent"), "{stderr}");
    assert!(stderr.contains("exited during startup"), "{stderr}");
    assert!(!mosaico::pty::read_all_metadata()
        .into_iter()
        .any(|metadata| metadata.agent == "missing-provider-role"));
    assert!(!Store::open(&home.store_path())
        .unwrap()
        .list_running_sessions()
        .unwrap()
        .into_iter()
        .any(|session| session.agent_slug == "missing-provider-role"));
    stop_daemon(&home);
}
