---
type: research-record
date: 2026-06-27
session: 01b0d1e8-aa18-4086-acad-237611e4c63d
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/01b0d1e8-aa18-4086-acad-237611e4c63d.jsonl
source_lines: 2424-2515
agent_attribution: main
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Root cause investigation: pre-registered diagnostic questions about daemon socket isolation and background processes → identified leaked test daemons as confound → clean re-runs revealed injection-path regression (not publish timeout) with structured verdict table"
captured_at: 2026-06-27T20:56:57Z
---

Root cause investigation: pre-registered diagnostic questions about daemon socket isolation and background processes → identified leaked test daemons as confound → clean re-runs revealed injection-path regression (not publish timeout) with structured verdict table

---

Assistant: You're right to call it out. Let me not defend that theory — the 500-event cap (or the filter's `limit`) does undercut it: a broad filter doesn't dump unbounded history, so "backlog saturates the socket" doesn't hold. I over-reached on the mechanism. Let me check two concrete things before saying anything else.

User: [Tool result]: chpwd:5: command not found: _iterm2_set_profile
=== does the test daemon socket key on TENEX_EDGE_HOME (i.e. is each test daemon isolated/cold)? ===
tests/daemon_integration/harness.rs:72:    pub(crate) fn sock(&self) -> PathBuf {
tests/daemon_integration/harness.rs:73:        self.dir.path().join("daemon.sock")
tests/daemon_integration/harness.rs:171:    if let Ok(stream) = UnixStream::connect(home.sock()) {
tests/daemon_integration/harness.rs:186:    while Instant::now() < deadline && home.sock().exists() {

=== any tenex-edge daemon processes currently running (could be reused as 'warm')? ===
pablofernandez   79081   0.0  0.1 442330928  28416   ??  S     9:50PM   0:11.07 /Users/pablofernandez/src/tenex-edge.ord/target/debug/tenex-edge __daemon
pablofernandez    3216   0.0  0.1 442306656  28256   ??  S     3:45PM   2:01.55 /Users/pablofernandez/.local/bin/tenex-edge __daemon
pablofernandez   98402   0.0  0.1 442330496  27872   ??  S    10:12PM   0:08.28 /Users/pablofernandez/src/tene

Assistant: Now I have something concrete, and it points at **me**, not the theory. Two findings:

1. Each test daemon's socket **is** keyed to its own tempdir home (`self.dir.path().join("daemon.sock")`) — so every test run spawns a *cold* daemon. No warm reuse. Fine.
2. But there are **leaked test daemons still running**: PIDs `79081` and `98402` are `target/debug/tenex-edge __daemon` from this worktree — orphaned from my repeated test runs that panicked before `stop_daemon`. They're **still connected to the shared test relay** (PID `3216` is your real daemon — I'm leaving that alone).

That's a real confound I created. My rebase test runs leaked daemons that keep hammering the shared relay; by the time I was re-running the failing test, several stale daemons were contending on that relay — which is a far more likely cause of a one-off publish-ACK timeout than anything in the ordinal code. Your instinct is probably correct.

Let me clean up my leaked daemons and re-run cleanly — master and rebase, apples-to-apples:

User: [Tool result]: === killing ONLY leaked .ord test daemons (NOT the user's ~/.local/bin daemon 3216) ===
=== remaining tenex-edge daemons ===
pablofernandez    3216   0.1  0.1 442308896  28272   ??  S     3:45PM   2:01.63 /Users/pablofernandez/.local/bin/tenex-edge __daemon

Assistant: Only your real daemon remains. Now the honest re-test on the rebase branch, clean:

User: [Tool result]: chpwd:5: command not found: _iterm2_set_profile
--- rebase clean run 1 ---
thread 'channels::launch_mentions::operator_kind9_to_offline_local_agent_spawns_and_injects' (6553525) panicked at tests/daemon_integration/channels/launch_mentions.rs:86:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 29 filtered out; finished in 27.19s
--- rebase clean run 2 ---
thread 'channels::launch_mentions::operator_kind9_to_offline_local_agent_spawns_and_injects' (6558921) panicked at tests/daemon_integration/channels/launch_mentions.rs:86:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 29 filtered out; finished in 26.86s
=== cleanup any leaked daemons from this run (keep 3216) ===
cleaned
Shell cwd was reset to /Users/pablofernandez/src/tenex-edge

Assistant: Important — the symptom **moved**. With my leaked daemons gone, `project_add` now succeeds; it now fails later at line 86 (`wait_for_injected_log`) — the agent spawns and goes alive, but the message never gets injected into its pane. So the `project_add` timeout I built my entire theory on **was** the leaked-daemon relay pollution. My flooding explanation was wrong, and I'm done asserting mechanisms I can't back.

Let me check one thing that determines whether the env is still contended — whether the test relay is shared with your live daemon:

User: [Tool result]: chpwd:5: command not found: _iterm2_set_profile
=== shared_nip29_relay_url ===
tests/daemon_integration/harness.rs:15:pub(crate) fn shared_nip29_relay_url() -> String {
tests/daemon_integration/harness.rs:15:pub(crate) fn shared_nip29_relay_url() -> String {
=== its value ===
pub(crate) fn shared_nip29_relay_url() -> String {
    static RELAY: OnceLock<TestRelay> = OnceLock::new();
    RELAY.get_or_init(TestRelay::start_nip29_relay).url.clone()
}

pub(crate) fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tenex-edge"))
=== user's daemon relay config ===
  "relays": ["wss://nip29.f7z.io"],
Shell cwd was reset to /Users/pablofernandez/src/tenex-edge

Assistant: That changes the picture again: `shared_nip29_relay_url()` spins up a **local, per-test-process relay** (`TestRelay::start_nip29_relay`), not your production relay (`nip29.f7z.io`). So your live daemon was never involved, and my "leaked daemons polluted the relay" explanation is also shaky (each test process gets its own relay on its own port).

I've now changed my story three times, which is exactly the BS you're pushing back on. Let me stop theorizing and get one clean fact: does **master** pass this exact test under clean conditions (no leaked daemons)? If master also fails clean, there's no regression to chase.

User: [Tool result]: === MASTER, clean, 2 runs ===
--- master run 1 ---
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 2.08s
--- master run 2 ---
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 1.90s
cleaned

Assistant: Now I have a clean, honest result. Master passes this test in ~2s (2/2). The rebase fails in ~27s (2/2) — but **not** at `project_add` anymore. With leaked daemons gone, `project_add` succeeds; it now fails at the **message-injection** step: the offline agent spawns and goes alive with the correct pubkey, but the operator's kind:9 body never reaches its pane.

So let me set the record straight on what you asked:

## The aggregate REQs — and why my "blocks the publish" claim was wrong

**What they actually are** (the legitimate part): the daemon multiplexes reads (REQ subscriptions) and writes (EVENT publishes + their OK acks) over one WebSocket to the relay. #47 collapses ~43 narrow per-(project×kind) REQs into 3 broad ones (`#h`/`#p`/group-state). That part is real and correct.

**Why my blocking explanation was BS:** you're right. A relay caps a subscription's initial replay (the filter `limit`, default ~500), so a broad filter does **not** dump unbounded history that head-of-line-blocks publishes. The mechanism I described doesn't happen. I retract it.

**What actually misled me:** the `project_add` publish timeout I built the whole theory on was **my own mess** — I'd leaked test daemons (PIDs 79081/98402) from runs that panicked before cleanup. Once I killed them, the `project_add` timeout vanished entirely. (Also: the test uses a *local, per-process* relay, not your `nip29.f7z.io` daemon — so neither your daemon nor relay backlog was ever in play.)

## The corrected, clean status

| | master | rebase |
|---|---|---|
| result | pass, ~2s (2/2) | fail, ~27s (2/2) |
| where | — | line 86: agent spawns + goes alive, but operator's kind:9 **never injected** into its pane |

So there **is** a real, reproducible regression — but it's in the **spawn-on-mention → inbound-message-injection** path (which the feature's "room-aware inbound chat/mention routing on (pubkey, h)" commit rewrote), **not** subscriptions or publishing. I genuinely don't have the cause yet, and I'm not going to hand you another guess.

