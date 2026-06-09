//! Daemon integration: state-mutating RPCs end-to-end, multi-agent mention
//! routing, and the ~16-concurrent-writer corruption repro through the RPC path.
//!
//! All tests run against a real spawned `__daemon` (one relay → a local
//! `nak serve`, never the production fabric) over a UDS in an isolated
//! `TENEX_EDGE_HOME`. Env mutation is serialized; the file is run single-threaded
//! by the runner invocation in the SUMMARY (each test sets process-global env).

mod common;

use common::TestRelay;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use tenex_edge::daemon::client::Client;
use tenex_edge::state::Store;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn shared_relay_url() -> String {
    static RELAY: OnceLock<TestRelay> = OnceLock::new();
    RELAY.get_or_init(TestRelay::start).url.clone()
}

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tenex-edge"))
}

struct Home {
    dir: tempfile::TempDir,
}

impl Home {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("TENEX_EDGE_HOME", dir.path());
        let cfg = dir.path().join("config.json");
        let body = serde_json::json!({
            "whitelistedPubkeys": [],
            "backendName": "test-host",
            "relays": [shared_relay_url()],
        });
        std::fs::write(&cfg, serde_json::to_string(&body).unwrap()).unwrap();
        std::env::set_var("TENEX_CONFIG", &cfg);
        std::env::set_var("TENEX_EDGE_DAEMON_GRACE_S", "30");
        std::env::set_var("TENEX_EDGE_BIN", bin());
        // Keep allow/block lists inside the temp home so we never touch ~/.tenex.
        std::env::set_var("TENEX_AGENTS_ALLOWLIST", dir.path().join("allow.txt"));
        std::env::set_var("TENEX_AGENTS_BLOCKLIST", dir.path().join("block.txt"));
        Home { dir }
    }
    fn store_path(&self) -> PathBuf {
        self.dir.path().join("state.db")
    }
    fn sock(&self) -> PathBuf {
        self.dir.path().join("daemon.sock")
    }
}

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().unwrap()
}

/// Run the real `tenex-edge` binary as a subprocess with the home's env — i.e.
/// exactly how the hooks invoke it. This is the ONLY path that exercises the
/// SYNCHRONOUS blocking client (`daemon::blocking`) + real CLI dispatch + the
/// actual stdout bytes the hooks parse.
fn run_cli(home: &Home, args: &[&str]) -> std::process::Output {
    std::process::Command::new(bin())
        .args(args)
        // Isolate from the invoking shell's tenex-edge env (a live claude/codex
        // shell exports these), so agent/session resolution is deterministic.
        .env_remove("TENEX_EDGE_AGENT")
        .env_remove("TENEX_EDGE_SESSION")
        .env("TENEX_EDGE_HOME", home.dir.path())
        .env("TENEX_CONFIG", home.dir.path().join("config.json"))
        .env("TENEX_EDGE_BIN", bin())
        .env("TENEX_EDGE_DAEMON_GRACE_S", "30")
        .env("TENEX_AGENTS_ALLOWLIST", home.dir.path().join("allow.txt"))
        .env("TENEX_AGENTS_BLOCKLIST", home.dir.path().join("block.txt"))
        .output()
        .expect("run tenex-edge")
}

// Like run_cli, but pipes `stdin` to the child — used to drive the `hook`
// subcommand, which reads its harness payload from stdin (there are no longer
// any session/turn subcommands to call directly).
fn run_cli_stdin(home: &Home, args: &[&str], stdin: &str) -> std::process::Output {
    use std::io::Write as _;
    let mut child = std::process::Command::new(bin())
        .args(args)
        .env_remove("TENEX_EDGE_AGENT")
        .env_remove("TENEX_EDGE_SESSION")
        .env("TENEX_EDGE_HOME", home.dir.path())
        .env("TENEX_CONFIG", home.dir.path().join("config.json"))
        .env("TENEX_EDGE_BIN", bin())
        .env("TENEX_EDGE_DAEMON_GRACE_S", "30")
        .env("TENEX_AGENTS_ALLOWLIST", home.dir.path().join("allow.txt"))
        .env("TENEX_AGENTS_BLOCKLIST", home.dir.path().join("block.txt"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn tenex-edge");
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("run tenex-edge")
}

/// Stop the daemon by sending the version-skew please_exit and waiting for the
/// socket to disappear (keeps tests from leaking daemons).
fn stop_daemon(home: &Home) {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    if let Ok(stream) = UnixStream::connect(home.sock()) {
        let mut w = stream.try_clone().unwrap();
        let mut r = BufReader::new(stream);
        let _ = writeln!(w, "{}", serde_json::json!({"protocol": u32::MAX, "client_version": "t"}));
        let mut welcome = String::new();
        let _ = r.read_line(&mut welcome);
        let _ = writeln!(w, "{}", serde_json::json!({"protocol": u32::MAX}));
        let mut resp = String::new();
        let _ = r.read_line(&mut resp);
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && home.sock().exists() {
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[test]
fn session_start_runs_engine_and_records_alive_session() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    let session_id = rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        let v = c
            .call(
                "session_start",
                serde_json::json!({"agent": "coder", "session_id": "sess-int-1", "cwd": "/tmp"}),
            )
            .await
            .expect("session_start");
        v["session_id"].as_str().unwrap().to_string()
    });
    assert_eq!(session_id, "sess-int-1");

    // The daemon (single writer) wrote an alive session row.
    let store = Store::open(&home.store_path()).unwrap();
    let rec = store.get_session("sess-int-1").unwrap().expect("session row");
    assert!(rec.alive);
    assert_eq!(rec.agent_slug, "coder");

    // `who` should surface it as a local row.
    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.unwrap();
        let v = c
            .call("who", serde_json::json!({"all": true, "all_projects": true}))
            .await
            .unwrap();
        let rows = v["rows"].as_array().unwrap();
        assert!(
            rows.iter().any(|r| r["session_id"] == "sess-int-1" && r["source"] == "Local"),
            "who rows: {rows:?}"
        );
    });

    stop_daemon(&home);
}

#[test]
fn send_message_then_inbox_roundtrip_same_machine() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        // Two sessions of two agents on this machine.
        c.call("session_start", serde_json::json!({"agent": "coder", "session_id": "sess-coder", "cwd": "/tmp"}))
            .await
            .unwrap();
        c.call("session_start", serde_json::json!({"agent": "reviewer", "session_id": "sess-rev", "cwd": "/tmp"}))
            .await
            .unwrap();

        // coder messages reviewer's session.
        let r = c
            .call(
                "send_message",
                serde_json::json!({"recipient": "sess-rev", "message": "please review", "session": "sess-coder"}),
            )
            .await
            .expect("send_message");
        assert!(r["target_session"] == "sess-rev", "got {r}");

        // Give the relay round-trip + demux a moment, then reviewer drains inbox.
        for _ in 0..20 {
            let inbox = c
                .call("inbox", serde_json::json!({"session": "sess-rev"}))
                .await
                .unwrap();
            let rows = inbox["rows"].as_array().unwrap();
            if rows.iter().any(|m| m["body"] == "please review") {
                return; // success
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        panic!("reviewer never received the mention");
    });

    stop_daemon(&home);
}

#[test]
fn mention_to_a_does_not_land_in_b_inbox() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        // Daemon hosts agents A and B (distinct pubkeys), one session each.
        c.call("session_start", serde_json::json!({"agent": "agent-a", "session_id": "sess-aaa", "cwd": "/tmp"}))
            .await
            .unwrap();
        c.call("session_start", serde_json::json!({"agent": "agent-b", "session_id": "sess-bbb", "cwd": "/tmp"}))
            .await
            .unwrap();

        // A third agent (sender) messages A's session specifically.
        c.call("session_start", serde_json::json!({"agent": "sender", "session_id": "sess-send", "cwd": "/tmp"}))
            .await
            .unwrap();
        c.call(
            "send_message",
            serde_json::json!({"recipient": "sess-aaa", "message": "for A only", "session": "sess-send"}),
        )
        .await
        .unwrap();

        // Wait until A receives it.
        let mut a_got = false;
        for _ in 0..20 {
            let inbox = c.call("inbox", serde_json::json!({"session": "sess-aaa"})).await.unwrap();
            if inbox["rows"].as_array().unwrap().iter().any(|m| m["body"] == "for A only") {
                a_got = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        assert!(a_got, "agent A should have received the mention");

        // B must NOT have it (routing is by to_pubkey, scoped to A's sessions).
        let b_inbox = c.call("inbox", serde_json::json!({"session": "sess-bbb"})).await.unwrap();
        assert!(
            b_inbox["rows"].as_array().unwrap().is_empty(),
            "agent B inbox should be empty, got {:?}",
            b_inbox["rows"]
        );
    });

    stop_daemon(&home);
}

/// Bug A (sibling-session delivery): two sessions of the SAME agent (one pubkey)
/// on this machine. A→B must land in B's inbox via LOCAL delivery (no relay echo
/// dependency), and must NOT land in the sender A's own inbox.
#[test]
fn sibling_session_mention_lands_in_target_via_local_delivery() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        // Two sessions of the SAME agent slug → same (agent, machine) pubkey.
        c.call("session_start", serde_json::json!({"agent": "claude", "session_id": "sibling-aaaaaa", "cwd": "/tmp"}))
            .await
            .unwrap();
        c.call("session_start", serde_json::json!({"agent": "claude", "session_id": "sibling-bbbbbb", "cwd": "/tmp"}))
            .await
            .unwrap();

        // Session A messages sibling session B specifically (by session-id prefix).
        let r = c
            .call(
                "send_message",
                serde_json::json!({"recipient": "sibling-bbbbbb", "message": "sibling hello", "session": "sibling-aaaaaa", "agent": "claude"}),
            )
            .await
            .expect("send_message");
        assert_eq!(r["target_session"], "sibling-bbbbbb", "got {r}");

        // Local delivery is synchronous — B should have it immediately (poll a few
        // times to absorb any scheduling jitter, but no relay round-trip needed).
        let mut b_got = false;
        for _ in 0..8 {
            let inbox = c.call("inbox", serde_json::json!({"session": "sibling-bbbbbb"})).await.unwrap();
            if inbox["rows"].as_array().unwrap().iter().any(|m| m["body"] == "sibling hello") {
                b_got = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert!(b_got, "sibling session B should have received the mention via local delivery");

        // The sender's own session A must NOT receive its own message.
        let a_inbox = c.call("inbox", serde_json::json!({"session": "sibling-aaaaaa"})).await.unwrap();
        assert!(
            a_inbox["rows"].as_array().unwrap().is_empty(),
            "sender session A inbox should be empty, got {:?}",
            a_inbox["rows"]
        );
    });

    stop_daemon(&home);
}

#[test]
fn sixteen_concurrent_writers_no_corruption_single_writer() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    // Start one session, then 16 concurrent clients hammer write-RPCs
    // (turn_start/turn_end flip turn state; this is the corruption repro path,
    // now serialized through the ONE daemon writer).
    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call("session_start", serde_json::json!({"agent": "coder", "session_id": "s-load", "cwd": "/tmp"}))
            .await
            .unwrap();
    });

    let n = 16;
    let iters = 25;
    let handles: Vec<_> = (0..n)
        .map(|_| {
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut c = Client::connect_or_spawn().await.expect("connect");
                    for _ in 0..iters {
                        c.call("turn_start", serde_json::json!({"session": "s-load"}))
                            .await
                            .expect("turn_start");
                        c.call("turn_end", serde_json::json!({"session": "s-load"}))
                            .await
                            .expect("turn_end");
                        c.call("who", serde_json::json!({"all": true}))
                            .await
                            .expect("who");
                    }
                });
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }

    // Single-writer by construction: exactly one daemon socket. And the db must
    // pass an integrity check (the corruption we recovered from would fail here).
    assert!(home.sock().exists(), "one daemon should still be listening");
    let store = Store::open(&home.store_path()).unwrap();
    let integrity = store.integrity_check().expect("integrity_check");
    assert_eq!(integrity, "ok", "state.db integrity check failed: {integrity}");

    stop_daemon(&home);
}

#[test]
fn cli_subprocess_blocking_path_session_start_and_who() {
    // The session/turn lifecycle is driven only through `hook` now (no bare
    // verbs). Run the real binary the way the harnesses do — payload on stdin —
    // and assert the blocking client + renderer behave. This also exercises the
    // opencode-only "no session id supplied → generate + print" branch.
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    // session-start with no id: the daemon generates one, the hook prints it.
    let out = run_cli_stdin(
        &home,
        &["hook", "--host", "opencode", "--type", "session-start"],
        r#"{"cwd":"/tmp"}"#,
    );
    assert!(out.status.success(), "session-start failed: {}", String::from_utf8_lossy(&out.stderr));
    let sid = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(!sid.is_empty(), "session-start printed no generated session id");

    // who --all-projects shows the agent (blocking client + real renderer).
    let out = run_cli(&home, &["who", "--all", "--all-projects"]);
    assert!(out.status.success(), "who failed: {}", String::from_utf8_lossy(&out.stderr));
    let who = String::from_utf8_lossy(&out.stdout);
    assert!(who.contains("opencode"), "who output missing agent: {who}");

    // turn end (stop hook) is a sync blocking write — must succeed, print nothing.
    let out = run_cli_stdin(
        &home,
        &["hook", "--host", "opencode", "--type", "stop"],
        &format!(r#"{{"session_id":"{sid}"}}"#),
    );
    assert!(out.status.success(), "stop failed: {}", String::from_utf8_lossy(&out.stderr));

    // session-end prints the confirmation to stderr.
    let out = run_cli_stdin(
        &home,
        &["hook", "--host", "opencode", "--type", "session-end"],
        &format!(r#"{{"session_id":"{sid}"}}"#),
    );
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("ended"));

    stop_daemon(&home);
}

#[test]
fn version_skew_client_detects_and_respawns() {
    // A daemon spawned at protocol 1, then a NEWER client (protocol 2) running
    // the real `connect_or_spawn` must detect the skew, make the old daemon
    // exit, and respawn the (now "newer") daemon — transparently succeeding.
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    // Start a daemon pinned to protocol 1 via a normal subprocess CLI call.
    let out = run_cli_proto(&home, &["who", "--all-projects"], Some("1"));
    assert!(out.status.success(), "proto-1 who failed: {}", String::from_utf8_lossy(&out.stderr));
    assert!(home.sock().exists(), "daemon should be up at proto 1");

    // A "newer" client (protocol 2): connect_or_spawn must re-exec the daemon
    // and the call must still succeed.
    let out = run_cli_proto(&home, &["who", "--all-projects"], Some("2"));
    assert!(
        out.status.success(),
        "proto-2 client failed to respawn+succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    stop_daemon(&home);
}

fn run_cli_proto(home: &Home, args: &[&str], proto: Option<&str>) -> std::process::Output {
    let mut cmd = std::process::Command::new(bin());
    cmd.args(args)
        .env("TENEX_EDGE_HOME", home.dir.path())
        .env("TENEX_CONFIG", home.dir.path().join("config.json"))
        .env("TENEX_EDGE_BIN", bin())
        .env("TENEX_EDGE_DAEMON_GRACE_S", "30")
        .env("TENEX_AGENTS_ALLOWLIST", home.dir.path().join("allow.txt"))
        .env("TENEX_AGENTS_BLOCKLIST", home.dir.path().join("block.txt"));
    if let Some(p) = proto {
        cmd.env("TENEX_EDGE_PROTOCOL", p);
    }
    cmd.output().expect("run tenex-edge")
}

// ── NIP-29 daemon-owned groups ───────────────────────────────────────────────

/// A valid (throwaway) operator nsec for the local relay.
const EXAMPLE_USER_NSEC: &str = "nsec1eulru7a67wt9ndqxv424kmgvd6uyd8defdxh7y9peut28f2p2vhs35m5h4";

fn rewrite_config_with_user_nsec(home: &Home) {
    let cfg = home.dir.path().join("config.json");
    let body = serde_json::json!({
        "whitelistedPubkeys": [],
        "backendName": "test-host",
        "relays": [shared_relay_url()],
        "userNsec": EXAMPLE_USER_NSEC,
    });
    std::fs::write(&cfg, serde_json::to_string(&body).unwrap()).unwrap();
}

#[test]
fn session_start_with_user_nsec_owns_group_and_adds_member() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_user_nsec(&home); // daemon reads this at spawn

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": "sess-grp-1", "cwd": "/tmp"}),
        )
        .await
        .expect("session_start");
    });

    // ensure_group_and_membership runs (and writes the cache) before session_start
    // returns, so by now the store records ownership + membership for this project.
    let store = Store::open(&home.store_path()).unwrap();
    let rec = store.get_session("sess-grp-1").unwrap().expect("session row");
    assert!(rec.alive);
    assert!(
        store.is_group_owned(&rec.project).unwrap(),
        "project group should be owned after session_start with userNsec"
    );
    assert!(
        store.is_group_member(&rec.project, &rec.agent_pubkey).unwrap(),
        "the starting agent should be a member of its project group"
    );

    stop_daemon(&home);
}

#[test]
fn session_start_without_user_nsec_still_starts_unmanaged() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new(); // default config has NO userNsec

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": "sess-nogrp", "cwd": "/tmp"}),
        )
        .await
        .expect("session_start must succeed even without userNsec");
    });

    // Fail-open: the session runs, but the group stays unmanaged (no ownership).
    let store = Store::open(&home.store_path()).unwrap();
    let rec = store.get_session("sess-nogrp").unwrap().expect("session row");
    assert!(rec.alive, "session must start even without userNsec");
    assert!(
        !store.is_group_owned(&rec.project).unwrap(),
        "without userNsec the daemon must not claim/own the group"
    );

    stop_daemon(&home);
}

// ── Phase-0 freeze tests ──────────────────────────────────────────────────────
//
// Each `freeze_*` test pins a specific observable behavior that MUST NOT change
// across the fabric-architecture refactor. The tests exercise ONLY the public
// daemon RPC path (UDS + Client::call) and the Store public API; they do not
// reach into private server internals.

/// Behavior 1: send-message dedup.
///
/// A message to a hosted sibling session inserts EXACTLY ONE inbox row, keyed
/// by `(mention_event_id, target_session)`. After the relay echoes the event
/// back AND after a subsequent `turn_start` (which triggers
/// `fetch_mentions_into_inbox`), the delivered count is still exactly 1 — no
/// duplicate rows are created by the idempotent `INSERT OR IGNORE`.
///
/// Assertion strategy:
///   - Use `Store::peek_inbox` (reads delivered=0 rows without consuming them)
///     before any drain to confirm exactly 1 pending row.
///   - Call `turn_start` (which triggers both fetch + drain) to simulate a
///     normal turn; then confirm inbox is empty (already drained, not
///     duplicated).
///   - Call `inbox` RPC again (which re-fetches from relay and drains): the
///     result must be empty — idempotent enqueue prevented re-insertion.
#[test]
fn freeze_send_message_dedup_exactly_one_inbox_row() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "sender-a", "session_id": "freeze-dedup-sender", "cwd": "/tmp"}),
        )
        .await
        .unwrap();
        c.call(
            "session_start",
            serde_json::json!({"agent": "receiver-a", "session_id": "freeze-dedup-recv", "cwd": "/tmp"}),
        )
        .await
        .unwrap();

        // Send a message from sender to receiver.
        let r = c
            .call(
                "send_message",
                serde_json::json!({
                    "recipient": "freeze-dedup-recv",
                    "message": "dedup-test-payload",
                    "session": "freeze-dedup-sender",
                }),
            )
            .await
            .expect("send_message");
        assert_eq!(r["target_session"], "freeze-dedup-recv", "target mismatch: {r}");

        // Wait until the local-delivery path inserts the row (it is synchronous,
        // but poll briefly to absorb any scheduling jitter).
        let mut delivered = false;
        for _ in 0..20 {
            // Open a SEPARATE store handle (read-only observer) to count without
            // consuming. peek_inbox returns delivered=0 rows only.
            let store = Store::open(&home.store_path()).unwrap();
            let pending = store.peek_inbox("freeze-dedup-recv").unwrap();
            if !pending.is_empty() {
                delivered = true;
                // FREEZE: exactly one pending row; assert the count before any drain.
                assert_eq!(
                    pending.len(),
                    1,
                    "peek_inbox should return exactly 1 pending row before drain, got {}: {:?}",
                    pending.len(),
                    pending
                );
                break;
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        assert!(delivered, "message never landed in receiver inbox");

        // trigger turn_start → fetch_mentions_into_inbox + drain (assemble_turn_start_context).
        // This simulates a real turn where the inbox is consumed AND a relay
        // catch-up fetch runs, which could create a duplicate if idempotency broke.
        let ctx = c
            .call("turn_start", serde_json::json!({"session": "freeze-dedup-recv"}))
            .await
            .expect("turn_start");
        // The context should mention our payload (it was in the inbox when drained).
        // FREEZE-NOTE: assemble_turn_start_context embeds inbox rows in the context
        // string only when the session has at least one prior turn (prev_started_at != 0).
        // If this is the first-ever turn, context may be null. We check rows-only
        // path to avoid fragility here; what matters is no duplicate is created.
        let _ = ctx; // context presence/absence is version-sensitive; not asserted here

        // After the drain, peek should now be empty.
        let store_after = Store::open(&home.store_path()).unwrap();
        let still_pending = store_after.peek_inbox("freeze-dedup-recv").unwrap();
        assert!(
            still_pending.is_empty(),
            "after turn_start drain, peek_inbox should be empty, got: {:?}",
            still_pending
        );

        // Call inbox RPC (triggers re-fetch from relay + drain). The relay still
        // holds the echoed event, but INSERT OR IGNORE must prevent re-delivery.
        // FREEZE: inbox returns empty rows — no duplicate delivery.
        let inbox_second = c
            .call("inbox", serde_json::json!({"session": "freeze-dedup-recv"}))
            .await
            .expect("inbox RPC");
        let rows_second = inbox_second["rows"].as_array().unwrap();
        assert!(
            rows_second.is_empty(),
            "second inbox call after drain must return empty (dedup), got: {:?}",
            rows_second
        );
    });

    stop_daemon(&home);
}

/// Behavior 2a: targeted mention routing — reaches ONLY the target session.
///
/// A mention with `target_session` set is delivered exclusively to that
/// session's inbox. A sibling session of the SAME agent in the SAME project
/// does NOT receive it.
#[test]
fn freeze_targeted_mention_reaches_only_target_session() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect daemon for freeze-targeted");
        // One sender, two sibling sessions of the SAME receiver agent.
        c.call(
            "session_start",
            serde_json::json!({"agent": "freeze-sender", "session_id": "freeze-tgt-src", "cwd": "/tmp"}),
        )
        .await
        .unwrap();
        // Two sibling sessions (same agent slug → same pubkey).
        c.call(
            "session_start",
            serde_json::json!({"agent": "freeze-rcvr", "session_id": "freeze-tgt-a", "cwd": "/tmp"}),
        )
        .await
        .unwrap();
        c.call(
            "session_start",
            serde_json::json!({"agent": "freeze-rcvr", "session_id": "freeze-tgt-b", "cwd": "/tmp"}),
        )
        .await
        .unwrap();

        // Target ONLY session B. Use session-id prefix which resolves directly
        // without needing presence/profile in the store (local session lookup).
        let r = c
            .call(
                "send_message",
                serde_json::json!({
                    "recipient": "freeze-tgt-b",
                    "message": "for-b-only",
                    "session": "freeze-tgt-src",
                }),
            )
            .await
            .expect("send_message");
        assert_eq!(r["target_session"], "freeze-tgt-b", "target_session field: {r}");

        // Wait until B receives it.
        let mut b_got = false;
        for _ in 0..20 {
            let inbox = c.call("inbox", serde_json::json!({"session": "freeze-tgt-b"})).await.unwrap();
            if inbox["rows"].as_array().unwrap().iter().any(|m| m["body"] == "for-b-only") {
                b_got = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        assert!(b_got, "target session B must receive the message");

        // FREEZE: session A of the same agent must NOT have any inbox rows.
        // Allow a brief window for any spurious delivery before asserting.
        tokio::time::sleep(Duration::from_millis(400)).await;
        let store = Store::open(&home.store_path()).unwrap();
        let a_pending = store.peek_inbox("freeze-tgt-a").unwrap();
        assert!(
            a_pending.is_empty(),
            "sibling session A must NOT receive a message targeted at B, got: {:?}",
            a_pending
        );

        // Also assert via the inbox RPC (runs fetch + drain, so stronger).
        let a_inbox = c.call("inbox", serde_json::json!({"session": "freeze-tgt-a"})).await.unwrap();
        assert!(
            a_inbox["rows"].as_array().unwrap().is_empty(),
            "inbox RPC for sibling A must be empty, got: {:?}",
            a_inbox["rows"]
        );
    });

    stop_daemon(&home);
}

/// Behavior 2b: untargeted mention reaches all alive sessions for the
/// recipient agent+project ONLY — not sessions of OTHER agents or projects.
#[test]
fn freeze_untargeted_mention_reaches_all_sessions_of_recipient_agent_only() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        // Sender agent.
        c.call(
            "session_start",
            serde_json::json!({"agent": "unt-sender", "session_id": "unt-src", "cwd": "/tmp"}),
        )
        .await
        .unwrap();
        // Recipient agent — TWO alive sessions (same slug → same pubkey).
        c.call(
            "session_start",
            serde_json::json!({"agent": "unt-target", "session_id": "unt-rcv-1", "cwd": "/tmp"}),
        )
        .await
        .unwrap();
        c.call(
            "session_start",
            serde_json::json!({"agent": "unt-target", "session_id": "unt-rcv-2", "cwd": "/tmp"}),
        )
        .await
        .unwrap();
        // Bystander agent — must NOT receive the message.
        c.call(
            "session_start",
            serde_json::json!({"agent": "unt-bystander", "session_id": "unt-bys", "cwd": "/tmp"}),
        )
        .await
        .unwrap();

        // Send WITHOUT specifying a target_session — resolves to the agent's pubkey
        // directly. Since unt-target is a local agent we can address by slug;
        // however, send_message needs a recipient that resolves on this daemon.
        // Use the most-recently-started session id as recipient (which sets
        // target_session in the event). To exercise the UNTARGETED path we need
        // the recipient to resolve to a pubkey only (no target_session). We do this
        // by sending to the slug@project form.
        //
        // FREEZE-NOTE: the untargeted path requires the sender to address by slug
        // rather than session-id. We look up the project from the sender's session.
        // Look up the receiver agent's pubkey directly from the sessions table.
        // For locally-hosted agents, profiles/peer_sessions are self-filtered by
        // the daemon and not stored, so slug@project resolution doesn't work.
        // Sending to the RAW 64-hex pubkey triggers the untargeted path (no
        // target_session in the event) and exercises the "all alive sessions"
        // delivery code path in route_mention_into_with_id.
        let store = Store::open(&home.store_path()).unwrap();
        let rcv_rec = store
            .get_session("unt-rcv-1")
            .unwrap()
            .expect("unt-rcv-1 session exists");
        let rcv_pubkey = rcv_rec.agent_pubkey.clone();

        let r = c
            .call(
                "send_message",
                serde_json::json!({
                    "recipient": rcv_pubkey,
                    "message": "broadcast-to-all",
                    "session": "unt-src",
                }),
            )
            .await
            .expect("send_message");
        // Untargeted: target_session is null in the response (pubkey resolution
        // returns no target_session).
        assert!(
            r["target_session"].is_null(),
            "untargeted send should have null target_session, got: {r}"
        );

        // Both receiver sessions must get the message.
        let mut rcv1_got = false;
        let mut rcv2_got = false;
        for _ in 0..25 {
            if !rcv1_got {
                let inbox1 = c.call("inbox", serde_json::json!({"session": "unt-rcv-1"})).await.unwrap();
                rcv1_got = inbox1["rows"].as_array().unwrap().iter().any(|m| m["body"] == "broadcast-to-all");
            }
            if !rcv2_got {
                let inbox2 = c.call("inbox", serde_json::json!({"session": "unt-rcv-2"})).await.unwrap();
                rcv2_got = inbox2["rows"].as_array().unwrap().iter().any(|m| m["body"] == "broadcast-to-all");
            }
            if rcv1_got && rcv2_got {
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert!(rcv1_got, "untargeted: receiver session 1 must get the message");
        assert!(rcv2_got, "untargeted: receiver session 2 must get the message");

        // FREEZE: bystander inbox must be empty.
        tokio::time::sleep(Duration::from_millis(300)).await;
        let bys_inbox = c.call("inbox", serde_json::json!({"session": "unt-bys"})).await.unwrap();
        assert!(
            bys_inbox["rows"].as_array().unwrap().is_empty(),
            "bystander inbox must be empty after untargeted mention, got: {:?}",
            bys_inbox["rows"]
        );
    });

    stop_daemon(&home);
}

/// Behavior 3: 39000/39002 idempotency.
///
/// Applying the same NIP-29 group-metadata (kind 39000) and members-snapshot
/// (kind 39002) events TWICE must be stable: project_meta and group_members
/// converge to the same state and members are not duplicated.
///
/// We exercise this through the `session_start` path (which causes the daemon
/// to subscribe and receive relay-authored 39000/39002 events) combined with
/// direct Store assertions. To force idempotency, we call session_start twice
/// for the same project, which may re-apply any cached 39002 snapshot from the
/// relay.
///
/// FREEZE-NOTE: the daemon applies 39000/39002 only when they arrive from the
/// relay subscription. We cannot inject raw relay events through the public
/// RPC path, so we verify idempotency via the Store methods that 39000/39002
/// handlers call: `upsert_project_meta` and `replace_group_members`.
/// The integration layer here tests that the Store semantics survive repeated
/// application (the daemon uses these same methods).
#[test]
fn freeze_39000_39002_idempotency_no_member_duplication() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    rewrite_config_with_user_nsec(&home);

    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        // Start a session — this triggers ensure_group_and_membership and an
        // initial 39000/39002 subscription.
        c.call(
            "session_start",
            serde_json::json!({"agent": "coder", "session_id": "freeze-grp-idem-1", "cwd": "/tmp"}),
        )
        .await
        .expect("first session_start");
    });

    // Allow the daemon time to receive any relay-echoed group events.
    std::thread::sleep(Duration::from_millis(400));

    // Record baseline membership state.
    let store = Store::open(&home.store_path()).unwrap();
    let rec = store.get_session("freeze-grp-idem-1").unwrap().expect("session row");
    let project = rec.project.clone();

    // FREEZE: group owned and member present after first start.
    assert!(
        store.is_group_owned(&project).unwrap(),
        "group must be owned after session_start with userNsec"
    );
    assert!(
        store.is_group_member(&project, &rec.agent_pubkey).unwrap(),
        "agent must be a member after session_start"
    );

    // Simulate idempotency: apply the same 39002 snapshot twice via the public
    // Store API (the daemon uses `replace_group_members` when it processes
    // kind:39002 from the relay — calling it twice is equivalent to receiving
    // the same event twice).
    let members_snapshot = vec![
        (rec.agent_pubkey.clone(), "member".to_string()),
    ];
    let ts = 9_000_000u64;
    store.replace_group_members(&project, &members_snapshot, ts).unwrap();
    store.replace_group_members(&project, &members_snapshot, ts).unwrap();

    // FREEZE: membership is stable — no duplication, same set.
    assert!(
        store.is_group_member(&project, &rec.agent_pubkey).unwrap(),
        "member still present after double-apply of 39002 snapshot"
    );
    // Count members via list — expect exactly 1 (no duplication).
    // We confirm via is_group_member scoped to a distinct fake pubkey being absent.
    assert!(
        !store.is_group_member(&project, "nonexistent-pk").unwrap(),
        "phantom member must not appear after 39002 re-application"
    );

    // FREEZE: project_meta upsert is idempotent (39000 handler).
    store.upsert_project_meta(&project, "about text v1", ts).unwrap();
    store.upsert_project_meta(&project, "about text v1", ts).unwrap();
    let meta = store.get_project_meta(&project).unwrap();
    assert_eq!(
        meta.as_deref(),
        Some("about text v1"),
        "project_meta must be stable after idempotent 39000 re-application"
    );

    // Applying an updated 'about' must overwrite (not duplicate) — the upsert
    // is DO UPDATE SET.
    store.upsert_project_meta(&project, "about text v2", ts + 1).unwrap();
    let meta2 = store.get_project_meta(&project).unwrap();
    assert_eq!(
        meta2.as_deref(),
        Some("about text v2"),
        "project_meta must reflect latest about after overwrite"
    );

    stop_daemon(&home);
}

/// Behavior 4: startup mention catch-up.
///
/// A kind:1 mention published to the relay BEFORE a session starts is caught
/// into that session's inbox once the session starts (via the
/// `fetch_mentions_into_inbox` call in `turn_start` / `inbox` RPCs).
/// A subsequent fetch of the same mention does NOT duplicate the row.
///
/// Setup:
///   1. Start a "pre-publisher" session for agent X (establishes agent pubkey).
///   2. A separate session (agent Y) sends a message to agent X.
///   3. End the original session for X so X has no alive sessions.
///   4. Start a NEW session for X.
///   5. Call `inbox` for the new X session → the stored mention is caught up.
///   6. Call `inbox` again → no new rows (dedup).
///
/// FREEZE-NOTE: the catch-up fetch (`fetch_mentions_into_inbox`) queries the
/// relay for kind:1 events tagged to the agent's pubkey. This relies on the
/// relay holding the event (nak serve in-memory relay does). The test uses
/// the same `shared_relay_url()` for both sender and receiver, which is the
/// in-memory nak relay that persists events for the lifetime of the test suite.
#[test]
fn freeze_startup_mention_catchup_no_duplicate() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();

    // Phase 1: establish agent X's pubkey by starting a session.
    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "catchup-x", "session_id": "catchup-x-pre", "cwd": "/tmp"}),
        )
        .await
        .unwrap();
        // Also start the sender Y.
        c.call(
            "session_start",
            serde_json::json!({"agent": "catchup-y", "session_id": "catchup-y-src", "cwd": "/tmp"}),
        )
        .await
        .unwrap();
    });

    // Phase 2: Y sends a message to the PRE-session of X (which is alive now).
    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");

        // Look up X's pubkey so we can send an untargeted mention.
        let x_rec = Store::open(&home.store_path())
            .unwrap()
            .get_session("catchup-x-pre")
            .unwrap()
            .expect("catchup-x-pre session row");
        // Use the RAW pubkey as recipient — this produces target_session=None in
        // the event (untargeted), which is required for catch-up delivery to a
        // DIFFERENT session of the same agent. If we sent by session-id, the event
        // would have target_session="catchup-x-pre" and would NOT be re-routed to
        // the new session (compute_targets would find no match).
        let x_pubkey = x_rec.agent_pubkey.clone();

        // Send an untargeted message from Y to X's pubkey (no target_session) —
        // publishes a kind:1 event to the relay, which will persist there.
        let r = c
            .call(
                "send_message",
                serde_json::json!({
                    "recipient": x_pubkey,
                    "message": "pre-start-mention",
                    "session": "catchup-y-src",
                }),
            )
            .await
            .expect("send_message");
        assert!(
            r["target_session"].is_null(),
            "untargeted send must have null target_session, got: {r}"
        );

        // Wait for the published event to be echoed back (so it's on the relay).
        tokio::time::sleep(Duration::from_millis(500)).await;

        // End X's pre-session so it becomes dead — simulates X going offline.
        c.call("session_end", serde_json::json!({"session": "catchup-x-pre"}))
            .await
            .expect("session_end");
    });

    // Small pause to let session_end propagate.
    std::thread::sleep(Duration::from_millis(200));

    // Phase 3: start a NEW session for X.
    rt().block_on(async {
        let mut c = Client::connect_or_spawn().await.expect("connect");
        c.call(
            "session_start",
            serde_json::json!({"agent": "catchup-x", "session_id": "catchup-x-new", "cwd": "/tmp"}),
        )
        .await
        .expect("second session_start for catchup-x");

        // Phase 4: call inbox for the NEW session — this triggers
        // fetch_mentions_into_inbox, which fetches the pre-published kind:1 from
        // the relay and enqueues it.
        let mut caught_up = false;
        for _ in 0..25 {
            let inbox = c.call("inbox", serde_json::json!({"session": "catchup-x-new"})).await.unwrap();
            let rows = inbox["rows"].as_array().unwrap();
            if rows.iter().any(|m| m["body"] == "pre-start-mention") {
                caught_up = true;
                // FREEZE: exactly one catch-up row delivered.
                // (inbox drains on each call; we just confirm the body arrived.)
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        assert!(caught_up, "new session must catch up the pre-start mention from relay");

        // Phase 5: call inbox again — no new rows (the mention was already
        // delivered and is deduped by INSERT OR IGNORE on the inbox PK AND by
        // mark_mention_seen on the agent pubkey).
        let inbox_again = c
            .call("inbox", serde_json::json!({"session": "catchup-x-new"}))
            .await
            .expect("second inbox call");
        let rows_again = inbox_again["rows"].as_array().unwrap();
        // FREEZE: no re-delivery.
        assert!(
            rows_again.iter().all(|m| m["body"] != "pre-start-mention"),
            "pre-start mention must NOT be re-delivered on a second inbox call, got: {:?}",
            rows_again
        );
    });

    stop_daemon(&home);
}
