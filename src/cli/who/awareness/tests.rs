use super::*;
use crate::state::{RelayEvent, Status, Store};

const NOW: u64 = 1_000;
/// The viewer's machine. Test peers share this host (so they render bare
/// `@slug`); the remote-peer test deliberately uses a different host.
const LOCAL_HOST: &str = "tower";

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
fn status(
    store: &Store,
    pubkey: &str,
    slug: &str,
    channel: &str,
    title: &str,
    busy: bool,
    ts: u64,
) {
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
        .upsert_profile(&pubkey, from_slug, from_slug, LOCAL_HOST, false, 1)
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
    // Opaque ids (`h-*`, `session-*`) distinct from human names so the
    // assertions below double as proof the raw channel_h never leaks.
    chan(
        &store,
        "h-core",
        "tenex-edge",
        "Agent coordination substrate",
        "",
    );
    chan(
        &store,
        "h-aware",
        "awareness",
        "Channel awareness hook",
        "h-core",
    );
    chan(
        &store,
        "h-ciflake",
        "ci-flake",
        "runner trust-cache failures",
        "h-aware",
    );
    // A real session room: opaque-id `name` that defaulted to its own id, nested
    // directly under the project root (`parent = h-core`) — so it reads as
    // unnamed (not a root) and is a top-level branch surfaced as an "other channel".
    chan(&store, "session-a9f2", "session-a9f2", "", "h-core");
    members(&store, "h-aware", &["pk-codex", "pk-claude"]);
    members(&store, "h-ciflake", &["pk-a", "pk-b"]);
    members(&store, "session-a9f2", &["pk-other"]);

    // Self (codex) and a peer (claude) are both live in #awareness; an unrelated
    // unnamed session room is active via its own member's status.
    status(
        &store,
        "pk-codex",
        "codex",
        "h-aware",
        "Designing channel awareness injection",
        true,
        995,
    );
    status(
        &store,
        "pk-claude",
        "claude",
        "h-aware",
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

    let block =
        render_awareness_snapshot(&store, "h-aware", NOW, "codex", "pk-codex", LOCAL_HOST).unwrap();

    assert_has(&block, "[tenex-edge] Fabric context");
    assert_has(
        &block,
        "Project: tenex-edge -- Agent coordination substrate",
    );
    // Channel is the project-RELATIVE path (root prefix dropped) + description.
    assert_has(&block, "Channel: awareness -- Channel awareness hook");
    assert_has(
        &block,
        "- @codex (you) - Designing channel awareness injection",
    );
    assert_has(&block, "- @claude - Tracing current status delta behavior");
    // Subchannel referenced by NAME, description after `--`.
    assert_has(
        &block,
        "- #ci-flake -- runner trust-cache failures [2 members]",
    );
    // Unnamed session room: labelled by its live work title, no id.
    assert_has(&block, "- Investigating duplicate session rooms [1 member]");
    // The opaque channel_h ids must never appear in agent-facing text.
    assert_lacks(&block, "h-core");
    assert_lacks(&block, "h-aware");
    assert_lacks(&block, "h-ciflake");
    assert_lacks(&block, "session-a9f2");
    assert_lacks(&block, "joined");
    assert_lacks(&block, "left");
}

#[test]
fn fabric_view_renders_unmaterialized_root_by_its_slug() {
    let store = Store::open_memory().unwrap();
    // No kind:39000 record for "tenex-edge" (a root project often isn't cached as
    // a channel), but a live session publishes into it.
    status(
        &store,
        "pk-dev",
        "developer",
        "tenex-edge",
        "Fix duplicate group creation on launch",
        false,
        980,
    );
    let block = super::render_fabric_view(&store, "tenex-edge", NOW, "", "", LOCAL_HOST);
    // The root shows by its SLUG, not by a session's work title, and not "(unnamed)".
    assert_has(&block, "Project: tenex-edge");
    assert_has(&block, "Channel: tenex-edge");
    assert_lacks(&block, "(unnamed channel)");
    // Members still render from live status even with no kind:39002 roster.
    assert_has(
        &block,
        "- @developer - Fix duplicate group creation on launch · idle",
    );
    // The `who` fabric view carries no injected-context header.
    assert_lacks(&block, "[tenex-edge] Fabric context");
}

#[test]
fn materialized_root_with_slug_id_renders_named_not_unnamed() {
    let store = Store::open_memory().unwrap();
    // The real-world shape (mirrors live ~/.tenex-edge state.db): a ROOT project's
    // NIP-29 group id IS its slug, and the relay defaults the kind:39000 `name` to
    // that same id — so `channel_h == name` with `parent` empty. This must read as
    // NAMED, even though session rooms use the identical `name == id` shape.
    chan(
        &store,
        "tenex-edge",
        "tenex-edge",
        "Agent coordination substrate",
        "",
    );
    // A child session room under it, with the same `name == id` shape but a parent
    // → genuinely unnamed, surfaced by its live work title.
    chan(&store, "session-x1", "session-x1", "", "tenex-edge");
    status(
        &store,
        "pk-dev",
        "developer",
        "session-x1",
        "Investigating duplicate session rooms",
        true,
        980,
    );

    let block = super::render_fabric_view(&store, "tenex-edge", NOW, "", "", LOCAL_HOST);
    // Root: named by its slug + about, never the unnamed placeholder.
    assert_has(
        &block,
        "Project: tenex-edge -- Agent coordination substrate",
    );
    assert_lacks(&block, "(unnamed channel)");
    // Child session room: labelled by its work title, never its raw id.
    assert_has(&block, "Investigating duplicate session rooms");
    assert_lacks(&block, "session-x1");
}

#[test]
fn new_agent_block_surfaces_only_agents_created_in_window() {
    let roster = vec![
        // created before the cursor → already known, not announced.
        ("old".to_string(), Some("stale helper".to_string()), 500u64),
        // created within (since, now] → newly available.
        (
            "writer".to_string(),
            Some("drafts posts".to_string()),
            950u64,
        ),
        // newly available, no byline.
        ("qa".to_string(), None, 960u64),
        // created in the future relative to now → ignored.
        ("future".to_string(), None, 2_000u64),
    ];
    let block = super::new_agent_block(&roster, 900, NOW).expect("two new agents in window");
    assert_has(
        &block,
        "New agents available (invite with `tenex-edge invite <slug>`):",
    );
    assert_has(&block, "- @writer - drafts posts");
    assert_has(&block, "- @qa");
    assert_lacks(&block, "old");
    assert_lacks(&block, "future");

    // Nothing created in the window → no section at all.
    assert!(super::new_agent_block(&roster, 1_000, NOW).is_none());
}

#[test]
fn remote_peer_is_host_qualified_local_peer_is_bare() {
    let store = Store::open_memory().unwrap();
    chan(&store, "h-aware", "awareness", "", "");
    members(&store, "h-aware", &["pk-local", "pk-remote"]);
    // Local peer shares the viewer's host (status() stamps "tower" == LOCAL_HOST).
    status(
        &store,
        "pk-local",
        "scout",
        "h-aware",
        "reviewing",
        true,
        960,
    );
    // Remote peer: a same-fabric agent on a different machine. Its `@slug@host`
    // form is exactly the token an agent would type to address it.
    store
        .upsert_profile("pk-remote", "developer", "developer", "laptop", false, 1)
        .unwrap();
    store
        .upsert_status(&Status {
            pubkey: "pk-remote".to_string(),
            channel_h: "h-aware".to_string(),
            slug: "developer".to_string(),
            title: "porting the resolver".to_string(),
            activity: String::new(),
            busy: true,
            last_seen: 965,
            updated_at: 965,
            expiration: NOW + 90,
        })
        .unwrap();

    // Snapshot path (member_lines).
    let snap =
        render_awareness_snapshot(&store, "h-aware", NOW, "codex", "pk-codex", LOCAL_HOST).unwrap();
    assert_has(&snap, "- @scout - reviewing");
    assert_has(&snap, "- @developer@laptop - porting the resolver");

    // Delta path (changed_member_lines) carries the same host-qualified form.
    let delta = render_awareness_update_since_check(
        &store,
        900,
        "h-aware",
        NOW,
        Some("pk-codex"),
        LOCAL_HOST,
    )
    .unwrap();
    assert_has(&delta, "- @scout - reviewing");
    assert_has(&delta, "- @developer@laptop - porting the resolver");
}

#[test]
fn update_renders_state_activity_and_omits_unchanged_sessions() {
    let store = Store::open_memory().unwrap();
    chan(&store, "h-aware", "awareness", "", "");
    chan(
        &store,
        "h-ciflake",
        "ci-flake",
        "Runner issue isolated",
        "h-aware",
    );
    // A sibling session room directly under the root → a top-level "other channel".
    chan(&store, "session-a9f2", "session-a9f2", "", "h-aware");
    members(&store, "h-aware", &["pk-claude"]);
    members(&store, "h-ciflake", &["pk-a", "pk-b"]);

    // Peer claude changed after the cursor (960 > 900).
    status(
        &store,
        "pk-claude",
        "claude",
        "h-aware",
        "Found the stale routing scope after channel switch",
        true,
        960,
    );
    // A subchannel and another channel saw status changes too.
    status(&store, "pk-a", "a", "h-ciflake", "fixing runner", true, 975);
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
        "h-aware",
        "claude",
        "The stale scope read is in turn_check.",
        970,
    );

    let block = render_awareness_update_since_check(
        &store,
        900,
        "h-aware",
        NOW,
        Some("pk-old"),
        LOCAL_HOST,
    )
    .unwrap();

    assert_has(&block, "[tenex-edge] Fabric updates since your last check");
    assert_has(
        &block,
        "- @claude - Found the stale routing scope after channel switch",
    );
    assert_has(&block, "- #ci-flake -- Runner issue isolated [2 members]");
    // Unnamed session room labelled by its live work title, never its id.
    assert_has(&block, "- other channel changed [1 member]");
    assert_has(&block, "Activity in #awareness:");
    assert_has(
        &block,
        "[@claude, just now] The stale scope read is in turn_check.",
    );
    assert_lacks(&block, "h-aware");
    assert_lacks(&block, "h-ciflake");
    assert_lacks(&block, "session-a9f2");
    assert_lacks(&block, "joined");
    assert_lacks(&block, "left");
}

#[test]
fn update_activity_excludes_viewers_own_chat() {
    let store = Store::open_memory().unwrap();
    chan(&store, "h-aware", "awareness", "", "");
    // The viewer (codex) authored a chat; it must not echo back to them.
    chat(
        &store,
        "chat-self",
        "h-aware",
        "codex",
        "did you validate it with real usage?",
        960,
    );
    chat(
        &store,
        "chat-other",
        "h-aware",
        "claude",
        "I validated it through the real hook.",
        970,
    );

    let block = render_awareness_update_since_check(
        &store,
        900,
        "h-aware",
        NOW,
        Some("pk-codex"),
        LOCAL_HOST,
    )
    .expect("other activity should still render");

    assert_has(&block, "Activity in #awareness:");
    assert_has(
        &block,
        "[@claude, just now] I validated it through the real hook.",
    );
    assert_lacks(&block, "did you validate it with real usage?");
}

#[test]
fn other_active_channels_use_status_titles_without_repeating_old_activity() {
    let store = Store::open_memory().unwrap();
    chan(&store, "h-aware", "awareness", "", "");
    // Session room directly under the root `h-aware` → unnamed (labelled by its
    // live work title) and a top-level branch surfaced as an "other channel".
    chan(&store, "session-a9f2", "session-a9f2", "", "h-aware");
    status(
        &store,
        "pk-codex",
        "codex",
        "session-a9f2",
        "Investigating duplicate session rooms",
        true,
        980,
    );

    let block =
        render_awareness_update_since_check(&store, 900, "h-aware", NOW, None, LOCAL_HOST).unwrap();
    // Unnamed session room labelled by its work title; the opaque id never shows.
    assert_has(&block, "- Investigating duplicate session rooms [1 member]");
    assert_lacks(&block, "session-a9f2");

    // No new status since 990 → nothing to repeat.
    let later = render_awareness_update_since_check(&store, 990, "h-aware", NOW, None, LOCAL_HOST);
    assert!(
        later.is_none(),
        "old active channel state must not repeat without new activity; got: {later:?}"
    );
}

#[test]
fn other_active_channels_are_scoped_to_this_project() {
    let store = Store::open_memory().unwrap();
    // Opaque ids distinct from human names (a `name == id` channel reads as
    // unnamed). THIS project: root `nmp` with a top-level branch `epic123`.
    chan(&store, "h-nmp", "nmp", "", "");
    chan(&store, "h-epic123", "epic123", "", "h-nmp");
    status(&store, "pk-a", "a", "h-epic123", "planning the epic", true, 980);
    // A DIFFERENT project: its own root `other-proj`, also active.
    chan(&store, "h-other", "other-proj", "", "");
    status(&store, "pk-b", "b", "h-other", "unrelated work", true, 980);
    // An orphan room whose ancestry can't be traced to any root (parent
    // un-materialized) → must be dropped, never leaked.
    chan(&store, "h-orphan", "orphan", "", "ghost-parent");
    status(&store, "pk-c", "c", "h-orphan", "ghost work", true, 980);

    // Viewer sits on the project root `nmp`.
    let block = super::render_fabric_view(&store, "h-nmp", NOW, "", "", LOCAL_HOST);
    // Our own top-level branch shows…
    assert_has(&block, "- #epic123 [1 member]");
    // …but the other project's root and the untraceable orphan never do.
    assert_lacks(&block, "other-proj");
    assert_lacks(&block, "unrelated work");
    assert_lacks(&block, "orphan");
    assert_lacks(&block, "ghost work");
}

#[test]
fn other_channels_exclude_the_branch_the_viewer_is_in() {
    let store = Store::open_memory().unwrap();
    let now = 20_000;
    let recent = now - 3 * 60 * 60;
    let stale = now - 4 * 60 * 60 - 1;
    chan(&store, "h-nmp", "nmp", "", "");
    chan(&store, "h-epic123", "epic123", "", "h-nmp");
    chan(&store, "h-epic999", "epic999", "", "h-nmp");
    chan(&store, "h-old", "old", "", "h-nmp");
    // A deeper room under the SIBLING branch epic999 — not a top-level branch, so
    // it must not surface as an "other channel" (only epic999 itself does).
    chan(&store, "h-e999deep", "e999-deep", "", "h-epic999");
    members(&store, "h-epic999", &["pk-a"]);
    status(
        &store,
        "pk-a",
        "a",
        "h-epic999",
        "sibling work",
        true,
        recent,
    );
    status(&store, "pk-b", "b", "h-e999deep", "deep work", true, recent);
    status(&store, "pk-old", "old", "h-old", "old work", true, stale);

    // Viewer is inside the epic123 branch.
    let block = super::render_fabric_view(&store, "h-epic123", now, "", "", LOCAL_HOST);
    assert_has(&block, "Other active channels, last 4h:");
    // The sibling top-level branch shows; the viewer's own branch does not (the
    // `Channel:` header names it, but it is never a "[N member]" channel line)…
    assert_has(&block, "- #epic999 [1 member]");
    assert_lacks(&block, "#epic123 [");
    assert_lacks(&block, "#old");
    // …and the room nested under epic999 is not a top-level "other channel".
    assert_lacks(&block, "deep work");
    assert_lacks(&block, "e999-deep");
}

#[test]
fn appeared_member_without_work_text_is_not_announced() {
    let store = Store::open_memory().unwrap();
    chan(&store, "child", "Channel awareness hook", "", "");
    status(&store, "pk-empty", "empty", "child", "", false, 980);

    let block = render_awareness_update_since_check(&store, 900, "child", NOW, None, LOCAL_HOST);
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

    let block =
        render_awareness_snapshot(&store, "child", NOW, "codex", "pk-codex", LOCAL_HOST).unwrap();
    assert_has(&block, "- @claude - Tracing current status delta behavior");
}
