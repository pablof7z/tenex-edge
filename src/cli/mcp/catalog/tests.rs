use super::*;

fn tool<'a>(tools: &'a [Value], name: &str) -> &'a Value {
    tools
        .iter()
        .find(|tool| tool["name"] == name)
        .unwrap_or_else(|| panic!("missing tool {name}"))
}

#[test]
fn catalog_contains_agent_coordination_tools_without_legacy_names() {
    let tools = list();
    for name in [
        "mosaico.skill",
        "mosaico.wait",
        "mosaico.channel_join",
        "mosaico.channel_send",
        "mosaico.dispatch",
        "mosaico.my_session",
    ] {
        tool(&tools, name);
    }
    for name in ["mosaico.who", "mosaico.channels_join", "mosaico.chat_write"] {
        assert!(tools.iter().all(|candidate| candidate["name"] != name));
    }
}

#[test]
fn wait_schema_exposes_ambient_and_correlated_forms() {
    let tools = list();
    let wait = tool(&tools, "mosaico.wait");
    assert_eq!(wait["annotations"]["readOnlyHint"], true);
    assert_eq!(wait["inputSchema"]["required"], json!(["timeout_seconds"]));
    for property in ["timeout_seconds", "channels", "from", "session"] {
        assert!(wait["inputSchema"]["properties"].get(property).is_some());
    }

    let send = tool(&tools, "mosaico.channel_send");
    assert_eq!(
        send["inputSchema"]["properties"]["wait_seconds"]["type"],
        "integer"
    );
}
