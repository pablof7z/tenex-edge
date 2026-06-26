use super::*;
use crate::session::{Harness, PeerStatusObservation, SessionObservation};
use crate::state::{ChatLogRow, Store};

fn register_local(
    store: &Store,
    slug: &str,
    pubkey: &str,
    project: &str,
    session: &str,
    ts: u64,
) -> String {
    store
        .register_or_reassert_session(&SessionObservation {
            agent_slug: slug.to_string(),
            agent_pubkey: pubkey.to_string(),
            project: project.to_string(),
            host: "laptop".to_string(),
            rel_cwd: String::new(),
            harness: Harness::ClaudeCode,
            harness_session_id: Some(session.to_string()),
            resume_id: None,
            tmux_pane: None,
            watch_pid: None,
            observed_at: ts,
        })
        .unwrap()
        .session_id
        .as_str()
        .to_string()
}

fn seed_title(store: &Store, session: &str, title: &str, ts: u64) {
    let turn = store.start_turn(session, ts).unwrap().unwrap();
    store
        .seed_title_if_empty(session, turn.turn_id, title, ts)
        .unwrap();
}

fn record_peer(store: &Store, slug: &str, pubkey: &str, project: &str, title: &str, ts: u64) {
    store
        .upsert_profile(pubkey, slug, "tower", false, ts)
        .unwrap();
    store
        .record_peer_status(&PeerStatusObservation {
            agent_pubkey: pubkey.to_string(),
            agent_slug: slug.to_string(),
            project: project.to_string(),
            host: "tower".to_string(),
            rel_cwd: String::new(),
            title: title.to_string(),
            activity: String::new(),
            busy: true,
            emitted_at: ts,
            observed_at: ts,
        })
        .unwrap();
}

fn record_chat(store: &Store, id: &str, project: &str, from: &str, body: &str, ts: u64) {
    record_chat_from_session(store, id, project, from, &format!("sid-{from}"), body, ts);
}

fn record_chat_from_session(
    store: &Store,
    id: &str,
    project: &str,
    from: &str,
    from_session: &str,
    body: &str,
    ts: u64,
) {
    store
        .record_chat(&ChatLogRow {
            chat_event_id: id.to_string(),
            from_pubkey: format!("pk-{from}"),
            from_slug: from.to_string(),
            host: "host".to_string(),
            project: project.to_string(),
            body: body.to_string(),
            created_at: ts,
            from_session: from_session.to_string(),
            mentioned_session: String::new(),
        })
        .unwrap();
}

fn group(store: &Store, id: &str, name: &str, parent: &str) {
    store.upsert_group_metadata(id, name, parent, 1).unwrap();
}

fn member(store: &Store, project: &str, pubkey: &str) {
    store
        .upsert_group_member(project, pubkey, "member", 1)
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
    group(&store, "tenex-edge", "Core repo", "");
    store
        .upsert_project_meta("tenex-edge", "Agent coordination substrate", 1)
        .unwrap();
    group(&store, "child", "Channel awareness hook", "tenex-edge");
    group(
        &store,
        "ci-flake",
        "Debugging runner trust-cache failures",
        "child",
    );
    group(
        &store,
        "session-a9f2",
        "Investigating duplicate session rooms",
        "",
    );
    for (project, pubkey) in [
        ("child", "pk-codex"),
        ("child", "pk-claude"),
        ("ci-flake", "pk-a"),
        ("ci-flake", "pk-b"),
        ("session-a9f2", "pk-other"),
    ] {
        member(&store, project, pubkey);
    }

    let me = register_local(&store, "codex", "pk-codex", "child", "sid-codex", 990);
    seed_title(&store, &me, "Designing channel awareness injection", 995);
    record_peer(
        &store,
        "claude",
        "pk-claude",
        "child",
        "Tracing current status delta behavior",
        996,
    );
    record_chat(
        &store,
        "chat-other",
        "session-a9f2",
        "claude",
        "semantic ping",
        997,
    );

    let block = render_awareness_snapshot(&store, "child", 1_000, "codex", "pk-codex").unwrap();

    assert_has(&block, "[tenex-edge] Fabric context");
    assert_has(
        &block,
        "Project: tenex-edge -- Agent coordination substrate",
    );
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
fn update_renders_state_activity_and_omits_gone_sessions() {
    let store = Store::open_memory().unwrap();
    group(&store, "child", "Channel awareness hook", "");
    group(&store, "ci-flake", "Runner issue isolated", "child");
    for (project, pubkey) in [
        ("child", "pk-claude"),
        ("child", "pk-old"),
        ("ci-flake", "pk-a"),
        ("ci-flake", "pk-b"),
        ("session-a9f2", "pk-other"),
    ] {
        member(&store, project, pubkey);
    }
    let old = register_local(&store, "old", "pk-old", "child", "sid-old", 800);
    store.end_session(&old, 950).unwrap();
    record_peer(
        &store,
        "claude",
        "pk-claude",
        "child",
        "Found the stale routing scope after channel switch",
        960,
    );
    record_chat(
        &store,
        "chat-child",
        "child",
        "claude",
        "The stale scope read is in turn_check.",
        970,
    );
    record_chat(
        &store,
        "chat-sub",
        "ci-flake",
        "claude",
        "subchannel changed",
        975,
    );
    record_chat(
        &store,
        "chat-other",
        "session-a9f2",
        "codex",
        "other channel changed",
        980,
    );

    let block =
        render_awareness_update_since_check(&store, 900, "child", 1_000, Some(&old)).unwrap();

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
    assert_lacks(&block, "@old");
    assert_lacks(&block, "joined");
    assert_lacks(&block, "left");
}

#[test]
fn update_activity_does_not_echo_current_session_prompt() {
    let store = Store::open_memory().unwrap();
    group(&store, "child", "Channel awareness hook", "");
    let me = register_local(&store, "codex", "pk-codex", "child", "sid-me", 900);
    record_chat_from_session(
        &store,
        "chat-self",
        "child",
        "operator",
        &me,
        "did you validate it with real usage?",
        960,
    );
    record_chat(
        &store,
        "chat-other",
        "child",
        "claude",
        "I validated it through the real hook.",
        970,
    );

    let block = render_awareness_update_since_check(&store, 900, "child", 1_000, Some(&me))
        .expect("other activity should still render");

    assert_has(&block, "Activity in #child:");
    assert_has(&block, "[@claude, just now] I validated it through the real hook.");
    assert_lacks(&block, "did you validate it with real usage?");
    assert_lacks(&block, "@operator");
}

#[test]
fn other_active_channels_use_status_titles_without_repeating_old_activity() {
    let store = Store::open_memory().unwrap();
    group(&store, "child", "Channel awareness hook", "");
    group(&store, "session-a9f2", "session-a9f2", "");
    record_peer(
        &store,
        "codex",
        "pk-codex",
        "session-a9f2",
        "Investigating duplicate session rooms",
        980,
    );

    let block = render_awareness_update_since_check(&store, 900, "child", 1_000, None).unwrap();
    assert_has(
        &block,
        "- #session-a9f2 -- Investigating duplicate session rooms [1 member]",
    );

    let later = render_awareness_update_since_check(&store, 990, "child", 1_000, None);
    assert!(
        later.is_none(),
        "old active channel state must not repeat without new semantic activity; got: {later:?}"
    );
}

#[test]
fn appeared_member_without_work_text_is_not_announced() {
    let store = Store::open_memory().unwrap();
    group(&store, "child", "Channel awareness hook", "");
    record_peer(&store, "empty", "pk-empty", "child", "", 980);

    let block = render_awareness_update_since_check(&store, 900, "child", 1_000, None);
    assert!(
        block.is_none(),
        "appearance without title/activity should not become joined/noise; got: {block:?}"
    );
}

#[test]
fn snapshot_includes_live_peer_even_when_roster_is_not_hydrated() {
    let store = Store::open_memory().unwrap();
    group(&store, "child", "Channel awareness hook", "");
    record_peer(
        &store,
        "claude",
        "pk-claude",
        "child",
        "Tracing current status delta behavior",
        980,
    );

    let block = render_awareness_snapshot(&store, "child", 1_000, "codex", "pk-codex").unwrap();
    assert_has(&block, "- @claude - Tracing current status delta behavior");
}
