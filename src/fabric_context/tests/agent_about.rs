use super::*;

#[test]
fn members_are_relay_roster_backed_and_local_agents_are_labeled() {
    let store = seed_store();
    let rec = session(&store);
    store
        .replace_agent_roster(&crate::state::AgentRoster {
            backend_pubkey: "backend".into(),
            host: "laptop".into(),
            slug: "helper".into(),
            use_criteria: "For testing".into(),
            channels: vec!["root".into()],
            updated_at: 2,
        })
        .unwrap();

    let text = render_fabric_context(&store, input(Some(&rec), "root", 0, 100, true))
        .expect("context should render");
    assert!(text.contains("<available-agents>"));
    assert!(text.contains("<agent ref=\"@helper\" about=\"For testing\""));
    assert!(!text.contains("<agents>"));
    assert!(text.contains("<member ref=\"@coder\""));

    let empty = Store::open_memory().unwrap();
    empty.upsert_channel("solo", "solo", "", "", 1).unwrap();
    empty
        .upsert_profile(SELF_PK, "coder", "coder", "laptop", false, 1)
        .unwrap();
    let solo = session_record(&empty, "solo", "solo");
    let text = render_fabric_context(&empty, input(Some(&solo), "solo", 0, 100, true)).unwrap();
    assert!(text.contains("<workspace name=\"solo\" channel=\"solo\""));
    assert!(!text.contains("<channel name=\"#solo\""));
    assert!(!text.contains("<members>"), "got: {text}");
}

#[test]
fn available_agent_about_is_compact_and_bounded() {
    let store = seed_store();
    let rec = session(&store);
    let long_about = format!("Routes\\nreview work {}", "carefully ".repeat(40));
    store
        .replace_agent_roster(&crate::state::AgentRoster {
            backend_pubkey: "backend".into(),
            host: "laptop".into(),
            slug: "helper".into(),
            use_criteria: long_about.clone(),
            channels: vec!["root".into()],
            updated_at: 2,
        })
        .unwrap();

    let text = render_fabric_context(&store, input(Some(&rec), "root", 0, 100, true))
        .expect("context should render");
    let expected = crate::agent_about::for_injection(&long_about);
    assert!(
        text.contains(&format!("about=\"{expected}\"")),
        "got: {text}"
    );
    assert!(expected.chars().count() <= crate::agent_about::AGENT_ABOUT_MAX_CHARS);
}
