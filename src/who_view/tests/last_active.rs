use super::*;
use crate::state::RecordMessage;

fn record(store: &Store, id: &str, channel: &str, state: &str, created_at: u64) {
    store
        .record_message(&RecordMessage {
            message_id: id.to_string(),
            thread_id: channel.to_string(),
            channel_h: channel.to_string(),
            author_pubkey: "human-pk".to_string(),
            body: "hello".to_string(),
            created_at,
            direction: "inbound".to_string(),
            sync_state: state.to_string(),
            native_event_id: Some(id.to_string()),
            error: None,
        })
        .unwrap();
}

#[test]
fn only_non_member_channels_show_last_accepted_activity() {
    let store = seed();
    record(&store, "compact-old", "compact", "accepted", 20);
    record(&store, "compact-failed", "compact", "failed", 99);
    record(&store, "joined-message", "joined", "accepted", 30);
    record(&store, "beta-message", "beta", "accepted", 40);
    let roots = vec!["alpha".to_string(), "beta".to_string()];
    let xml = render_agent_who(
        &store,
        AgentWhoInput {
            roots: &roots,
            self_name: "quill-peak-369-codex",
            self_pubkey: "self-pk",
            local_host: "laptop",
            backend_pubkey: "backend-pk",
            now: 140,
            headless: false,
            active_channels: &BTreeSet::from(["alpha".to_string()]),
            expanded_workspaces: &BTreeSet::from(["alpha".to_string()]),
        },
    )
    .unwrap();

    assert!(xml.contains(
        "<channel name=\"small-talk\" id=\"/alpha/small-talk\" about=\"Chit chat\" \
         members=\"1\" last-active=\"2 min ago\" />"
    ));
    assert!(!xml.contains("id=\"/alpha/planning\" about=\"Plan work\" members=\"1\" last-active="));
    assert!(!xml.contains("id=\"/beta\" members=\"1\" last-active="));
}

#[test]
fn non_member_channel_without_messages_omits_last_active() {
    let xml = render(false);
    assert!(xml.contains(
        "<channel name=\"small-talk\" id=\"/alpha/small-talk\" about=\"Chit chat\" \
         members=\"1\" />"
    ));
    assert!(!xml.contains("last-active="));
}
