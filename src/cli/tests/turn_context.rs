use super::*;
use crate::session::{Harness, SessionObservation};
use crate::state::{SessionRecord, Store};
use std::sync::Mutex;

/// Register a local session into `session_state` (daemon mints the canonical
/// id) and return it.
fn register_local(store: &Store, slug: &str, pubkey: &str, harness_sid: &str, ts: u64) -> String {
    let obs = SessionObservation {
        agent_slug: slug.to_string(),
        agent_pubkey: pubkey.to_string(),
        project: "proj".to_string(),
        host: "laptop".to_string(),
        rel_cwd: String::new(),
        harness: Harness::ClaudeCode,
        harness_session_id: Some(harness_sid.to_string()),
        resume_id: None,
        tmux_pane: None,
        watch_pid: None,
        observed_at: ts,
    };
    store
        .register_or_reassert_session(&obs)
        .unwrap()
        .session_id
        .as_str()
        .to_string()
}

/// Register a busy local session carrying a distilled title + activity line.
/// Appears at `reg_ts` (so a cursor after it sees a *change*, not an appear)
/// and the distill lands at `change_ts`.
#[allow(clippy::too_many_arguments)]
fn register_busy(
    store: &Store,
    slug: &str,
    pubkey: &str,
    harness_sid: &str,
    title: &str,
    activity: &str,
    reg_ts: u64,
    change_ts: u64,
) -> String {
    let id = register_local(store, slug, pubkey, harness_sid, reg_ts);
    let snap = store.start_turn(&id, change_ts).unwrap().unwrap();
    store
        .apply_distill_result(
            &id,
            snap.turn_id,
            snap.state_version,
            title,
            activity,
            change_ts,
        )
        .unwrap()
        .unwrap();
    id
}

/// Register a local session that opened and then finished a turn, so it is
/// idle but retains its title. Appears at `reg_ts`; the busy→idle change
/// lands at `change_ts`.
fn register_idle(
    store: &Store,
    slug: &str,
    pubkey: &str,
    harness_sid: &str,
    title: &str,
    reg_ts: u64,
    change_ts: u64,
) -> String {
    let id = register_local(store, slug, pubkey, harness_sid, reg_ts);
    let snap = store.start_turn(&id, change_ts).unwrap().unwrap();
    store
        .seed_title_if_empty(&id, snap.turn_id, title, change_ts)
        .unwrap()
        .unwrap();
    store.end_turn(&id, change_ts).unwrap().unwrap();
    id
}

/// Build a minimal alive SessionRecord for testing context assembly.
fn test_session(id: &str) -> SessionRecord {
    SessionRecord {
        session_id: id.to_string(),
        agent_slug: "coder".to_string(),
        agent_pubkey: "pk-coder".to_string(),
        project: "proj".to_string(),
        host: "laptop".to_string(),
        child_pid: None,
        watch_pid: None,
        created_at: 1,
        alive: true,
        rel_cwd: String::new(),
        channel: String::new(),
    }
}

/// turn_start returns None on a non-first turn with no chat and no peers.
#[test]
fn turn_start_context_returns_none_when_empty_non_first_turn() {
    let store = Store::open_memory().unwrap();
    let rec = test_session("sess-freeze-2");
    // No chat rows. Non-first turn (prev != 0). No peer sessions.
    let m = Mutex::new(store);

    let ctx = assemble_turn_start_context(&m, &rec, /* prev_turn_started_at */ 42);
    assert!(
        ctx.is_none(),
        "turn_start with no chat, non-first turn, no peers must return None; got: {ctx:?}"
    );
}

#[test]
fn first_turn_renders_awareness_snapshot_not_session_code() {
    let store = Store::open_memory().unwrap();
    let rec = test_session("sess-intro");
    let m = Mutex::new(store);

    let text = assemble_turn_start_context(&m, &rec, 0).expect("first-turn intro expected");
    assert!(
        text.contains("[tenex-edge] Fabric context"),
        "first turn should render fabric awareness; got: {text:?}"
    );
    assert!(
        text.contains("Channel: #proj"),
        "awareness should name the channel; got: {text:?}"
    );
    assert!(
        text.contains("@coder (you)"),
        "awareness should identify this agent; got: {text:?}"
    );
    assert!(
        !text.contains("[session"),
        "intro must not expose a session code; got: {text:?}"
    );
    assert!(
        !text.contains("@<codename>"),
        "intro must not teach codename mentions; got: {text:?}"
    );
}

/// turn_check returns None when there is no chat and delta_since=None (rate-limited).
#[test]
fn turn_check_context_returns_none_when_nothing_due() {
    let store = Store::open_memory().unwrap();
    let m = Mutex::new(store);
    let ctx = assemble_turn_check_context(&m, &test_session("sess-no-rows"), "laptop", None, 200);
    assert!(
        ctx.is_none(),
        "turn_check with no chat, no delta → None; got: {ctx:?}"
    );
}

/// Mid-turn delta: a sibling session's status change in the same project is
/// surfaced with its activity line; the viewer's own row is excluded.
#[test]
fn turn_check_delta_shows_siblings_with_activity_excludes_self() {
    let store = Store::open_memory().unwrap();
    store
        .upsert_profile("pk-sib", "sib", "laptop", false, 1)
        .unwrap();
    // Sibling registered before the cursor (10), then changed after it (180)
    // and is still live at now=200 → surfaces as a Changed delta.
    let sib_id = register_busy(
        &store,
        "sib",
        "pk-sib",
        "sess-sib",
        "Refactor tmux",
        "editing hooks.rs",
        10,
        180,
    );
    // The viewer's own session also changed — must NOT echo back.
    let me_id = register_busy(
        &store,
        "coder",
        "pk-coder",
        "sess-me",
        "My own work",
        "typing",
        10,
        180,
    );
    let m = Mutex::new(store);

    let text = assemble_turn_check_context(&m, &test_session(&me_id), "laptop", Some(50), 200)
        .expect("delta block expected when a sibling changed");
    assert!(
        text.contains("[tenex-edge] Fabric updates since your last check"),
        "awareness update header expected; got: {text:?}"
    );
    assert!(
        text.contains("@sib - Refactor tmux — editing hooks.rs"),
        "sibling activity expected as a member work line; got: {text:?}"
    );
    assert!(
        !text.contains("My own work"),
        "viewer's own status must be excluded; got: {text:?}"
    );
    assert!(
        !text.contains(&crate::util::session_codename(&sib_id)),
        "session code must not render as the primary identity; got: {text:?}"
    );
    assert!(
        !text.contains(sib_id.as_str()),
        "raw session id must not leak; got: {text:?}"
    );
    assert!(!text.contains("joined"), "got: {text:?}");
    assert!(!text.contains("left"), "got: {text:?}");
}

/// Mid-turn delta: a sibling that went idle renders with the `· idle` marker
/// so peers can see when someone stopped working.
#[test]
fn turn_check_delta_shows_idle_transition() {
    let store = Store::open_memory().unwrap();
    store
        .upsert_profile("pk-sib", "sib", "laptop", false, 1)
        .unwrap();
    // Sibling appeared before the cursor (10), then opened+finished a turn at
    // 180 → idle, title retained, still live at now=200 → Changed delta.
    register_idle(
        &store,
        "sib",
        "pk-sib",
        "sess-sib",
        "Refactor tmux",
        10,
        180,
    );
    let m = Mutex::new(store);

    let text = assemble_turn_check_context(&m, &test_session("sess-me"), "laptop", Some(50), 200)
        .expect("delta block expected for idle transition");
    assert!(
        text.contains("@sib - Refactor tmux · idle"),
        "idle marker expected in the member work line; got: {text:?}"
    );
    assert!(!text.contains("joined"), "got: {text:?}");
    assert!(!text.contains("left"), "got: {text:?}");
}

/// Repeated idle/end observations are liveness refreshes, not user-visible
/// status changes. They must not re-emit the same `title · idle` line.
#[test]
fn turn_check_delta_suppresses_repeated_idle_noop() {
    let store = Store::open_memory().unwrap();
    store
        .upsert_profile("pk-sib", "sib", "laptop", false, 1)
        .unwrap();
    let sib_id = register_idle(&store, "sib", "pk-sib", "sess-sib", "Refactor tmux", 10, 20);
    store.end_turn(&sib_id, 180).unwrap().unwrap();
    let m = Mutex::new(store);

    let text = assemble_turn_check_context(&m, &test_session("sess-me"), "laptop", Some(50), 200);
    assert!(
        text.is_none(),
        "unchanged idle session must not be emitted again; got: {text:?}"
    );
}

/// Repeated session-start/reassert observations refresh liveness and tmux
/// endpoint aliases, but identical public state is not a status delta.
#[test]
fn turn_check_delta_suppresses_identical_session_reassert() {
    let store = Store::open_memory().unwrap();
    store
        .upsert_profile("pk-sib", "sib", "laptop", false, 1)
        .unwrap();
    register_local(&store, "sib", "pk-sib", "sess-sib", 10);
    register_local(&store, "sib", "pk-sib", "sess-sib", 180);
    let m = Mutex::new(store);

    let text = assemble_turn_check_context(&m, &test_session("sess-me"), "laptop", Some(50), 200);
    assert!(
        text.is_none(),
        "identical session reassert must not be emitted as a change; got: {text:?}"
    );
}

fn chat_row(session_id: &str, eid: &str, created_at: u64) -> crate::state::ChatInboxRow {
    crate::state::ChatInboxRow {
        chat_event_id: eid.to_string(),
        target_session: session_id.to_string(),
        from_pubkey: "pk-chat".to_string(),
        from_slug: "chatter".to_string(),
        project: "proj".to_string(),
        body: "ambient chatter".to_string(),
        created_at,
        from_session: String::new(),
        mentioned_session: String::new(),
    }
}

fn direct_mention_row(session_id: &str, eid: &str, created_at: u64) -> crate::state::ChatInboxRow {
    let mut row = chat_row(session_id, eid, created_at);
    row.body = "please review this now".to_string();
    row.mentioned_session = session_id.to_string();
    row
}

/// Project chat is delta-gated: a row newer than the cursor surfaces once,
/// but a row older than the cursor (already shown earlier this turn) does
/// not re-emit on the next tool call. The peek never marks it delivered, so
/// without the cursor filter it would repeat on every PostToolUse.
#[test]
fn turn_check_chat_shown_once_not_per_tool_call() {
    let store = Store::open_memory().unwrap();
    // Arrived at 120, after the cursor (50) → surfaces on this check.
    store
        .enqueue_chat(&chat_row("sess-me", "chat-new", 120))
        .unwrap();
    let m = Mutex::new(store);

    let text = assemble_turn_check_context(&m, &test_session("sess-me"), "laptop", Some(50), 200)
        .expect("fresh chat past the cursor must surface");
    assert!(
        text.contains("[tenex-edge] Messages on #proj since your last check:"),
        "chat block expected; got: {text:?}"
    );

    // Next check's cursor has advanced past the row (since=150 > 120): the
    // same undelivered row must NOT re-emit.
    let text2 = assemble_turn_check_context(&m, &test_session("sess-me"), "laptop", Some(150), 200);
    assert!(
        text2.is_none(),
        "already-shown chat must not repeat once the cursor passes it; got: {text2:?}"
    );

    // The row is still undelivered (peek, not drain) so turn_start delivers it.
    let g = m.lock().unwrap();
    assert_eq!(g.peek_chat("sess-me").unwrap().len(), 1);
}

/// `delta_since = None` (rate-limited / not mid-turn) suppresses project chat
/// too, not just the sibling delta — chat is debounced, the inbox is not.
#[test]
fn turn_check_chat_suppressed_when_not_due() {
    let store = Store::open_memory().unwrap();
    store
        .enqueue_chat(&chat_row("sess-me", "chat-x", 120))
        .unwrap();
    let m = Mutex::new(store);

    let ctx = assemble_turn_check_context(&m, &test_session("sess-me"), "laptop", None, 200);
    assert!(
        ctx.is_none(),
        "chat must be suppressed when not due (no inbox to surface); got: {ctx:?}"
    );
}

/// Direct mentions are different from ambient chat: if no tmux pane is
/// available, the next hook must surface them even when the normal
/// awareness/delta rate-limit is closed. Once surfaced, they are marked
/// notified so tool hooks do not repeat them, but they remain undelivered
/// for tmux / turn-start prompt delivery.
#[test]
fn turn_check_direct_mentions_surface_without_delta_window_and_notify() {
    let store = Store::open_memory().unwrap();
    store
        .enqueue_chat(&direct_mention_row("sess-me", "mention-1", 120))
        .unwrap();
    let m = Mutex::new(store);

    let ctx = assemble_turn_check_context(&m, &test_session("sess-me"), "laptop", None, 200)
        .expect("direct mention must surface at the next available hook");
    assert!(
        ctx.contains("Incoming message mentioning this agent"),
        "direct mention must be surfaced as input addressed to the agent; got: {ctx:?}"
    );
    assert!(ctx.contains("please review this now"));

    let s = m.lock().unwrap();
    assert!(
        s.peek_unnotified_chat_mentions("sess-me")
            .unwrap()
            .is_empty(),
        "direct mention should be marked notified after hook fallback"
    );
    assert!(
        !s.peek_chat_mentions("sess-me").unwrap().is_empty(),
        "notified direct mention must remain undelivered for tmux/turn-start"
    );
    assert!(
        s.list_recently_delivered_chat_mentions("sess-me", 0)
            .unwrap()
            .is_empty(),
        "hook notification must not count as prompt delivery"
    );
}

/// `delta_since = None` (rate-limited / not mid-turn) suppresses the sibling
/// delta entirely, even when a sibling just changed.
#[test]
fn turn_check_delta_suppressed_when_not_due() {
    let store = Store::open_memory().unwrap();
    store
        .upsert_profile("pk-sib", "sib", "laptop", false, 1)
        .unwrap();
    register_busy(
        &store,
        "sib",
        "pk-sib",
        "sess-sib",
        "Refactor tmux",
        "",
        10,
        180,
    );
    let m = Mutex::new(store);

    let ctx = assemble_turn_check_context(&m, &test_session("sess-me"), "laptop", None, 200);
    assert!(
        ctx.is_none(),
        "no delta and no inbox → None when not due; got: {ctx:?}"
    );
}

fn view<'a>() -> EnvelopeView<'a> {
    EnvelopeView {
        from_slug: "codex",
        project: "tenex-edge",
        from_session: "sender-session-id",
        host: "",
        self_host: "my-box",
        subject: "NIP-29 group creation failing",
        branch: "features/oauth",
        commit: "a1b2c3d",
        dirty: 0,
        id: "01234567",
        sent_at: 1_000,
        now: 1_180, // +3 min
        body: "can you take a look?",
    }
}

#[test]
fn envelope_has_email_like_headers_then_body() {
    let out = format_envelope(&view());
    let lines: Vec<&str> = out.lines().collect();
    // Canonical sender identity: `codename (agent@host)`.
    assert_eq!(
        lines[0],
        format!(
            "From: {} (codex@my-box)",
            session_codename("sender-session-id")
        )
    );
    assert!(lines[1].starts_with("Date: ") && lines[1].ends_with("(3 min ago)"));
    assert_eq!(lines[2], "Subject: NIP-29 group creation failing");
    assert_eq!(lines[3], "Branch: features/oauth (a1b2c3d)");
    assert_eq!(lines[4], "ID: 01234567");
    assert_eq!(lines[5], "--");
    assert_eq!(lines[6], "can you take a look?");
}

#[test]
fn dirty_count_and_remote_host_annotate() {
    let mut v = view();
    v.dirty = 1;
    v.host = "prod-01.example.com";
    let out = format_envelope(&v);
    // Host (slugified) rides in the canonical `agent@host`; no `[remote:]` tag.
    assert!(out.contains("(codex@prod-01-example-com)"));
    assert!(out.contains("Branch: features/oauth (a1b2c3d) [1 file dirty]"));
    v.dirty = 3;
    assert!(format_envelope(&v).contains("[3 files dirty]"));
}

#[test]
fn subject_and_branch_lines_omitted_when_empty() {
    let mut v = view();
    v.subject = "";
    v.branch = "";
    let out = format_envelope(&v);
    assert!(!out.contains("Subject:"));
    assert!(!out.contains("Branch:"));
    // Same-host sender → no remote annotation.
    assert!(!out.contains("remote:"));
}
