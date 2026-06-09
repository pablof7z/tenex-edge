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
