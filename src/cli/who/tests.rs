use super::channel::channel_status_map;
use super::render::{render_who_once, render_who_plain, render_whoami};
use super::snapshot::{OtherProjectSummary, SpawnableRow, WhoRow, WhoSnapshot, WhoSource};
use super::*;
use crate::session::{Harness, PeerStatusObservation, SessionObservation};
use crate::util::session_codename;

fn strip_ansi(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Register a local session into `session_state` and return its minted canonical
/// id. The daemon mints the id, so callers capture it from the returned snapshot
/// rather than asserting a fixed string.
#[allow(clippy::too_many_arguments)]
fn register_local(
    store: &Store,
    slug: &str,
    pubkey: &str,
    project: &str,
    host: &str,
    rel_cwd: &str,
    harness_session_id: &str,
    observed_at: u64,
) -> String {
    let obs = SessionObservation {
        agent_slug: slug.to_string(),
        agent_pubkey: pubkey.to_string(),
        project: project.to_string(),
        host: host.to_string(),
        rel_cwd: rel_cwd.to_string(),
        harness: Harness::ClaudeCode,
        harness_session_id: Some(harness_session_id.to_string()),
        resume_id: None,
        tmux_pane: None,
        watch_pid: None,
        observed_at,
    };
    store
        .register_or_reassert_session(&obs)
        .unwrap()
        .session_id
        .as_str()
        .to_string()
}

/// Open a turn and seed a provisional title, so the local row carries a busy
/// title (the new model's equivalent of the deleted `set_agent_status`).
fn seed_busy_title(store: &Store, session_id: &str, title: &str, ts: u64) {
    let turn = store.start_turn(session_id, ts).unwrap().unwrap();
    store
        .seed_title_if_empty(session_id, turn.turn_id, title, ts)
        .unwrap()
        .unwrap();
}

/// Mirror a peer kind:30315 into `peer_session_state`.
#[allow(clippy::too_many_arguments)]
fn record_peer(
    store: &Store,
    pubkey: &str,
    slug: &str,
    project: &str,
    _native_session_id: &str,
    host: &str,
    rel_cwd: &str,
    title: &str,
    busy: bool,
    emitted_at: u64,
) {
    let obs = PeerStatusObservation {
        agent_pubkey: pubkey.to_string(),
        agent_slug: slug.to_string(),
        project: project.to_string(),
        host: host.to_string(),
        rel_cwd: rel_cwd.to_string(),
        title: title.to_string(),
        activity: String::new(),
        busy,
        emitted_at,
        observed_at: emitted_at,
    };
    store.record_peer_status(&obs).unwrap();
}

fn local_session(id: &str) -> crate::state::SessionRecord {
    crate::state::SessionRecord {
        session_id: id.to_string(),
        agent_slug: "coder".to_string(),
        agent_pubkey: "pk-coder".to_string(),
        project: "proj".to_string(),
        host: "laptop".to_string(),
        child_pid: Some(42),
        watch_pid: None,
        created_at: 1,
        alive: true,
        rel_cwd: String::new(),
        channel: String::new(),
    }
}

#[test]
fn who_snapshot_merges_local_and_peer_sessions() {
    let store = Store::open_memory().unwrap();
    // Local coder lives in session_state (the single source of truth).
    let coder_id = register_local(
        &store,
        "coder",
        "pk-coder",
        "proj",
        "laptop",
        "",
        "sid-coder",
        1_000,
    );
    // A peer echo of our own local session (same minted id) must be deduped.
    record_peer(
        &store, "pk-coder", "coder", "proj", &coder_id, "laptop", "", "", false, 1_000,
    );
    // A genuine remote peer on a different host.
    record_peer(
        &store,
        "pk-reviewer",
        "reviewer",
        "proj",
        "remote-session",
        "tower",
        "",
        "reviewing the patch",
        true,
        995,
    );

    // Daemon/viewer host is "laptop" → the local coder is same-machine; the
    // "tower" reviewer is a genuine remote.
    let snapshot = load_who_snapshot(&store, Some("proj"), 1_000, "laptop").unwrap();

    assert_eq!(snapshot.rows.len(), 2);
    let coder = snapshot
        .rows
        .iter()
        .find(|r| r.source == WhoSource::Local && r.slug == "coder")
        .expect("local coder row");
    let reviewer = snapshot
        .rows
        .iter()
        .find(|r| r.source == WhoSource::Peer && r.slug == "reviewer")
        .expect("peer reviewer row");
    // The self-echo (peer row mirroring our own canonical id) is hidden.
    assert!(!snapshot
        .rows
        .iter()
        .any(|r| r.source == WhoSource::Peer && r.session_id.as_str() == coder_id));

    // §8e same-host/remote: this machine's own session is NOT remote; a peer
    // on a different host IS.
    assert!(!coder.remote, "local session must never be remote");
    assert!(reviewer.remote, "tower peer must be remote vs laptop");

    let once = strip_ansi(&render_who_once(&snapshot));
    assert!(once.starts_with("proj\n\n"));
    // Host is shown for every agent now, including same-machine sessions. The
    // freshly-registered coder is idle (no turn opened yet).
    assert!(once.contains("coder (laptop) - idle"));
    assert!(!once.contains("[session"));
    assert!(!once.contains(&session_codename(&coder_id)));
    assert!(once.contains("reviewer (tower, remote) - reviewing the patch"));
}

#[test]
fn who_snapshot_uses_session_scoped_status_for_sibling_sessions() {
    let store = Store::open_memory().unwrap();
    // Two sibling sessions for the same agent, each with its own canonical id.
    let id_a = register_local(
        &store,
        "claude",
        "pk-claude",
        "proj",
        "laptop",
        "",
        "sid-a",
        1_000,
    );
    let id_b = register_local(
        &store,
        "claude",
        "pk-claude",
        "proj",
        "laptop",
        "",
        "sid-b",
        1_000,
    );
    // Each gets its own per-session title — proving status is session-scoped.
    seed_busy_title(&store, &id_a, "reading files", 1_000);
    seed_busy_title(&store, &id_b, "running tests", 1_000);

    let snapshot = load_who_snapshot(&store, Some("proj"), 1_000, "laptop").unwrap();
    let row_a = snapshot
        .rows
        .iter()
        .find(|r| r.session_id.as_str() == id_a)
        .expect("session-a row");
    let row_b = snapshot
        .rows
        .iter()
        .find(|r| r.session_id.as_str() == id_b)
        .expect("session-b row");
    assert_eq!(row_a.status, "reading files");
    assert_eq!(row_b.status, "running tests");
}

#[test]
fn who_snapshot_ignores_same_host_peer_echo_for_known_local_agent() {
    let store = Store::open_memory().unwrap();
    // A prior (now dead) local session for pk-claude is recorded in `sessions`,
    // so list_local_agent_pubkeys knows pk-claude is one of ours.
    let mut old = local_session("old-local");
    old.agent_slug = "claude".to_string();
    old.agent_pubkey = "pk-claude".to_string();
    old.alive = false;
    store.upsert_session(&old).unwrap();
    // The same identity arrives over the wire as a same-host peer echo.
    record_peer(
        &store,
        "pk-claude",
        "claude",
        "proj",
        "old-local",
        "laptop",
        "",
        "",
        false,
        1_000,
    );

    let snapshot = load_who_snapshot(&store, Some("proj"), 1_000, "laptop").unwrap();
    assert!(
        snapshot.rows.is_empty(),
        "same-host peer echo for our own local identity should be hidden"
    );
}

mod rendering;
mod session_pubkeys;

#[test]
fn same_host_peer_is_not_remote() {
    // A sibling agent (e.g. codex@) on the SAME laptop arrives as a peer row;
    // it must NOT be tagged remote (the bug being fixed).
    let store = Store::open_memory().unwrap();
    record_peer(
        &store,
        "pk-codex",
        "codex",
        "proj",
        "sib",
        "laptop",
        "worktree1",
        "",
        false,
        1_000,
    );
    let snap = load_who_snapshot(&store, Some("proj"), 1_000, "laptop").unwrap();
    let sib = snap
        .rows
        .iter()
        .find(|r| r.slug == "codex")
        .expect("sibling row");
    assert!(!sib.remote, "same-host peer must not be remote");
    assert_eq!(sib.rel_cwd, "worktree1");
    let once = strip_ansi(&render_who_once(&snap));
    assert!(
        once.contains("codex (laptop)"),
        "same-host peer shows its host"
    );
    assert!(
        !once.contains("remote"),
        "same-host peer must not be remote"
    );
    assert!(once.contains("[worktree1]"), "rel_cwd shown in bracket");
}

#[test]
fn root_rel_cwd_has_no_bracket() {
    let store = Store::open_memory().unwrap();
    // rel_cwd "." (project root) → no [dir] bracket.
    record_peer(
        &store, "pk-a", "a", "proj", "r", "tower", ".", "", false, 1_000,
    );
    let snap = load_who_snapshot(&store, Some("proj"), 1_000, "laptop").unwrap();
    let once = strip_ansi(&render_who_once(&snap));
    assert!(!once.contains("[.]"), "root cwd must not render a bracket");
    assert!(
        once.contains("a (tower, remote)"),
        "remote peer shows host plus a remote flag"
    );
}
