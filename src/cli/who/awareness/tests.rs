use super::*;
use crate::state::{RelayEvent, Status, Store};

const NOW: u64 = 1_000;

/// Materialize a kind:39000 channel.
fn chan(store: &Store, id: &str, name: &str, about: &str, parent: &str) {
    store.upsert_channel(id, name, about, parent, 1).unwrap();
}

/// Replace a channel's member roster (kind:39002).
fn members(store: &Store, project: &str, pubkeys: &[&str]) {
    let v: Vec<String> = pubkeys.iter().map(|s| s.to_string()).collect();
    store.replace_channel_members(project, &v, 1).unwrap();
}

/// Publish a live kind:30315 status for an agent in a channel.
fn status(store: &Store, pubkey: &str, slug: &str, channel: &str, title: &str, busy: bool, ts: u64) {
    store
        .upsert_profile(pubkey, slug, slug, "tower", false, 1)
        .unwrap();
    store
        .upsert_status(&Status {
            pubkey: pubkey.to_string(),
            channel_h: channel.to_string(),
            slug: slug.to_string(),
            title: title.to_string(),
            activity: String::new(),
            busy,
            last_seen: ts,
            updated_at: ts,
            expiration: NOW + 90,
        })
        .unwrap();
}

/// Append a kind:9 chat event to a channel's relay-event log.
fn chat(store: &Store, id: &str, channel: &str, from_slug: &str, body: &str, ts: u64) {
    let pubkey = format!("pk-{from_slug}");
    store
        .upsert_profile(&pubkey, from_slug, from_slug, "host", false, 1)
        .unwrap();
    store
        .insert_event(&RelayEvent {
            id: id.to_string(),
            kind: 9,
            pubkey,
            created_at: ts,
            channel_h: channel.to_string(),
            d_tag: String::new(),
            content: body.to_string(),
            tags_json: "[]".to_string(),
        })
        .unwrap();
}

fn assert_has(block: &str, needle: &str) {
    assert!(block.contains(needle), "missing {needle:?}; got: {block}");
}

fn assert_lacks(block: &str, needle: &str) {
    assert!(
        !block.contains(needle),
        "unexpected {needle:?}; got: {block}"
    );
}

#[test]
fn snapshot_renders_awareness_without_transport_events() {
    let store = Store::open_memory().unwrap();
    chan(
        &store,
        "tenex-edge",
        "Core repo",
        "Agent coordination substrate",
        "",
    );
    chan(&store, "child", "Channel awareness hook", "", "tenex-edge");
    chan(
        &store,
        "ci-flake",
        "Debugging runner trust-cache failures",
        "",
        "child",
    );
    chan(
        &store,
        "session-a9f2",
        "Investigating duplicate session rooms",
        "",
        "",
    );
    members(&store, "child", &["pk-codex", "pk-claude"]);
    members(&store, "ci-flake", &["pk-a", "pk-b"]);
    members(&store, "session-a9f2", &["pk-other"]);

    // Self (codex) and a peer (claude) are both live in #child; an unrelated
    // channel (session-a9f2) is active via its own member's status.
    status(
        &store,
        "pk-codex",
        "codex",
        "child",
        "Designing channel awareness injection",
        true,
        995,
    );
    status(
        &store,
        "pk-claude",
        "claude",
        "child",
        "Tracing current status delta behavior",
        true,
        996,
    );
    status(
        &store,
        "pk-other",
        "other",
        "session-a9f2",
        "Investigating duplicate session rooms",
        true,
        997,
    );

    let block = render_awareness_snapshot(&store, "child", NOW, "codex", "pk-codex").unwrap();

    assert_has(&block, "[tenex-edge] Fabric context");
    assert_has(&block, "Project: tenex-edge -- Agent coordination substrate");
    assert_has(
        &block,
        "Channel: #tenex-edge -- Core repo > #child -- Channel awareness hook",
    );
    assert_has(
        &block,
        "- @codex (you) - Designing channel awareness injection",
    );
    assert_has(&block, "- @claude - Tracing current status delta behavior");
    assert_has(
        &block,
        "- #ci-flake -- Debugging runner trust-cache failures [2 members]",
    );
    assert_has(
        &block,
        "- #session-a9f2 -- Investigating duplicate session rooms [1 member]",
    );
    assert_lacks(&block, "joined");
    assert_lacks(&block, "left");
}

#[test]
fn update_renders_state_activity_and_omits_unchanged_sessions() {
    let store = Store::open_memory().unwrap();
    chan(&store, "child", "Channel awareness hook", "", "");
    chan(&store, "ci-flake", "Runner issue isolated", "", "child");
    members(&store, "child", &["pk-claude"]);
    members(&store, "ci-flake", &["pk-a", "pk-b"]);

    // Peer claude changed after the cursor (960 > 900).
    status(
        &store,
        "pk-claude",
        "claude",
        "child",
        "Found the stale routing scope after channel switch",
        true,
        960,
    );
    // A subchannel and another channel saw status changes too.
    status(&store, "pk-a", "a", "ci-flake", "fixing runner", true, 975);
    status(
        &store,
        "pk-other",
        "other",
        "session-a9f2",
        "other channel changed",
        true,
        980,
    );
    chat(
        &store,
        "chat-child",
        "child",
        "claude",
        "The stale scope read is in turn_check.",
        970,
    );

    let block =
        render_awareness_update_since_check(&store, 900, "child", NOW, Some("pk-old")).unwrap();

    assert_has(&block, "[tenex-edge] Fabric updates since your last check");
    assert_has(
        &block,
        "- @claude - Found the stale routing scope after channel switch",
    );
    assert_has(&block, "- #ci-flake -- Runner issue isolated [2 members]");
    assert_has(&block, "- #session-a9f2 [1 member]");
    assert_has(&block, "Activity in #child:");
    assert_has(
        &block,
        "[@claude, just now] The stale scope read is in turn_check.",
    );
    assert_lacks(&block, "joined");
    assert_lacks(&block, "left");
}

#[test]
fn update_activity_excludes_viewers_own_chat() {
    let store = Store::open_memory().unwrap();
    chan(&store, "child", "Channel awareness hook", "", "");
    // The viewer (codex) authored a chat; it must not echo back to them.
    chat(
        &store,
        "chat-self",
        "child",
        "codex",
        "did you validate it with real usage?",
        960,
    );
    chat(
        &store,
        "chat-other",
        "child",
        "claude",
        "I validated it through the real hook.",
        970,
    );

    let block = render_awareness_update_since_check(&store, 900, "child", NOW, Some("pk-codex"))
        .expect("other activity should still render");

    assert_has(&block, "Activity in #child:");
    assert_has(
        &block,
        "[@claude, just now] I validated it through the real hook.",
    );
    assert_lacks(&block, "did you validate it with real usage?");
}

#[test]
fn other_active_channels_use_status_titles_without_repeating_old_activity() {
    let store = Store::open_memory().unwrap();
    chan(&store, "child", "Channel awareness hook", "", "");
    chan(&store, "session-a9f2", "session-a9f2", "", "");
    status(
        &store,
        "pk-codex",
        "codex",
        "session-a9f2",
        "Investigating duplicate session rooms",
        true,
        980,
    );

    let block = render_awareness_update_since_check(&store, 900, "child", NOW, None).unwrap();
    assert_has(
        &block,
        "- #session-a9f2 -- Investigating duplicate session rooms [1 member]",
    );

    // No new status since 990 → nothing to repeat.
    let later = render_awareness_update_since_check(&store, 990, "child", NOW, None);
    assert!(
        later.is_none(),
        "old active channel state must not repeat without new activity; got: {later:?}"
    );
}

#[test]
fn appeared_member_without_work_text_is_not_announced() {
    let store = Store::open_memory().unwrap();
    chan(&store, "child", "Channel awareness hook", "", "");
    status(&store, "pk-empty", "empty", "child", "", false, 980);

    let block = render_awareness_update_since_check(&store, 900, "child", NOW, None);
    assert!(
        block.is_none(),
        "appearance without title/activity should not become noise; got: {block:?}"
    );
}

#[test]
fn snapshot_includes_live_peer_even_when_roster_is_not_hydrated() {
    let store = Store::open_memory().unwrap();
    chan(&store, "child", "Channel awareness hook", "", "");
    status(
        &store,
        "pk-claude",
        "claude",
        "child",
        "Tracing current status delta behavior",
        true,
        980,
    );

    let block = render_awareness_snapshot(&store, "child", NOW, "codex", "pk-codex").unwrap();
    assert_has(&block, "- @claude - Tracing current status delta behavior");
}
