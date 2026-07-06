use crate::identity::*;

#[test]
fn load_or_create_with_command_seeds_default_command_only_on_creation() {
    let dir = tempfile::tempdir().unwrap();
    let seeded = vec![
        "claude".to_string(),
        "--agent".to_string(),
        "smith".to_string(),
    ];
    let a = load_or_create_with_command(dir.path(), "smith", 1, Some(seeded.clone())).unwrap();
    assert_eq!(a.commands.len(), 1);
    assert_eq!(a.commands[0].name, DEFAULT_COMMAND_NAME);
    assert_eq!(a.commands[0].argv.as_slice(), seeded.as_slice());

    // A second call with a different command must not overwrite the stored one.
    let other = vec![
        "claude".to_string(),
        "--agent".to_string(),
        "other".to_string(),
    ];
    let b = load_or_create_with_command(dir.path(), "smith", 2, Some(other)).unwrap();
    assert_eq!(b.commands.len(), 1);
    assert_eq!(b.commands[0].argv.as_slice(), seeded.as_slice());
    assert_eq!(a.pubkey_hex(), b.pubkey_hex());
}

#[test]
fn add_local_agent_sets_and_overwrites_default_command() {
    let dir = tempfile::tempdir().unwrap();
    let (a, _) = add_local_agent(
        dir.path(),
        "dev",
        Some(vec![
            "claude".into(),
            "--dangerously-skip-permissions".into(),
        ]),
        1,
    )
    .unwrap();
    assert_eq!(a.commands.len(), 1);
    assert_eq!(a.commands[0].name, DEFAULT_COMMAND_NAME);
    assert_eq!(
        a.commands[0].argv.as_slice(),
        &["claude", "--dangerously-skip-permissions"]
    );
    let (b, created) = add_local_agent(dir.path(), "dev", Some(vec!["codex".into()]), 2).unwrap();
    assert!(!created);
    assert_eq!(a.pubkey_hex(), b.pubkey_hex());
    assert_eq!(b.commands[0].argv.as_slice(), &["codex"]);
}

#[test]
fn add_local_agent_with_commands_persists_multiple_named_commands() {
    let dir = tempfile::tempdir().unwrap();
    let commands = vec![
        LaunchCommand::new("safe", vec!["claude".into()]).unwrap(),
        LaunchCommand::new(
            "full",
            vec!["claude".into(), "--dangerously-skip-permissions".into()],
        )
        .unwrap(),
    ];

    let (id, created) =
        add_local_agent_with_commands(dir.path(), "dev", commands.clone(), 1).unwrap();

    assert!(created);
    assert_eq!(id.commands, commands);
    let rows = list_local_agent_details(dir.path());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].commands, commands);
}

#[test]
fn list_local_agent_details_surfaces_pubkey_and_commands() {
    let dir = tempfile::tempdir().unwrap();
    let (a, _) = add_local_agent(dir.path(), "coder", None, 1).unwrap();
    add_local_agent(dir.path(), "dev", Some(vec!["codex".into()]), 1).unwrap();
    let rows = list_local_agent_details(dir.path());
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].slug, "coder");
    assert_eq!(rows[0].pubkey, a.pubkey_hex());
    assert!(rows[0].commands.is_empty());
    assert_eq!(rows[1].slug, "dev");
    assert_eq!(rows[1].commands[0].argv.as_slice(), &["codex"]);
}

#[test]
fn commands_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("agents")).unwrap();
    std::fs::write(
        dir.path().join("agents/dev.json"),
        r#"{"slug":"dev","secret_key":"0000000000000000000000000000000000000000000000000000000000000001","public_key":"","created_at":1,"commands":[{"name":"full","argv":["claude","--dangerously-skip-permissions"]}]}"#,
    )
    .unwrap();
    let agents = list_local_agents(dir.path());
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].0, "dev");
    assert_eq!(agents[0].1[0].name, "full");
    assert_eq!(
        agents[0].1[0].argv.as_slice(),
        &["claude", "--dangerously-skip-permissions"]
    );
    assert!(agents[0].2.is_none());
    assert!(agents[0].3.is_none());
}

#[test]
fn legacy_singular_command_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("agents")).unwrap();
    std::fs::write(
        dir.path().join("agents/dev.json"),
        r#"{"slug":"dev","secret_key":"0000000000000000000000000000000000000000000000000000000000000001","public_key":"","created_at":1,"command":["claude"]}"#,
    )
    .unwrap();

    let agents = list_local_agents(dir.path());
    assert_eq!(agents.len(), 1);
    assert!(agents[0].1.is_empty());
}
