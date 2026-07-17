use super::*;
use crate::test_env::EnvGuard;

fn write(path: &std::path::Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

#[test]
fn explicit_conflict_combination_persists_canonical_profile_binding() {
    let root = tempfile::tempdir().unwrap();
    let mosaico_home = root.path().join("mosaico");
    let codex_home = root.path().join(".codex");
    let mut env = EnvGuard::set("HOME", root.path());
    env.set_var("MOSAICO_HOME", &mosaico_home);
    env.set_var("MOSAICO_ISOLATED_HOME_OK", "1");
    env.set_var("CODEX_HOME", &codex_home);
    env.set_var("XDG_CONFIG_HOME", root.path().join(".config"));
    write(
        &mosaico_home.join("config.json"),
        r#"{"availableHarnesses":["claude","codex"]}"#,
    );
    write(
        &mosaico_home.join("harnesses.json"),
        r#"{
          "claude-pty":{"harness":"claude","transport":"pty"},
          "codex-pty":{"harness":"codex","transport":"pty"}
        }"#,
    );
    write(
        &codex_home.join("agents/writer.toml"),
        "name='writer'\ndescription='Writes'\ndeveloper_instructions='Write'",
    );
    write(
        &root.path().join(".claude/agents/writer.md"),
        "---\nname: writer\ndescription: Writes\n---\nWrite",
    );

    let selection = resolve_fresh_agent("writer-codex", root.path()).unwrap();

    assert_eq!(selection.slug, "writer");
    assert_eq!(
        selection.retired_advertisements,
        ["writer-claude", "writer-codex"]
    );
    let binding: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(mosaico_home.join("agents/writer.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(binding["slug"], "writer");
    assert_eq!(binding["harness"], "codex-pty");
    assert!(binding.get("profile").is_none());
}

#[test]
fn menu_rows_are_aligned_single_line_and_bounded() {
    let inventory = crate::agent_inventory::AgentInventory {
        agents: vec![
            crate::agent_inventory::AvailableAgent {
                slug: "writing-partner".into(),
                agent_slug: "writing-partner".into(),
                bundle: "claude-pty".into(),
                harness: crate::session::Harness::ClaudeCode,
                use_criteria: "Drafts\\nrevises   and publishes ".repeat(20),
                available_since: 0,
                source: crate::agent_inventory::AgentSource::Configured,
                persist_binding: false,
            },
            crate::agent_inventory::AvailableAgent {
                slug: "codex".into(),
                agent_slug: "codex".into(),
                bundle: "codex-pty".into(),
                harness: crate::session::Harness::Codex,
                use_criteria: String::new(),
                available_since: 0,
                source: crate::agent_inventory::AgentSource::Harness,
                persist_binding: false,
            },
        ],
        failures: Vec::new(),
    };

    let agents = ordered_agents(&inventory);
    let labels = menu_labels(&agents);

    assert_eq!(labels[0], "codex            Codex harness");
    assert!(!labels[1].contains('\n'));
    assert!(labels[1].ends_with('…'));
    assert!(labels[1].chars().count() <= MAX_NAME_CHARS + 2 + MAX_DETAIL_CHARS);
}
