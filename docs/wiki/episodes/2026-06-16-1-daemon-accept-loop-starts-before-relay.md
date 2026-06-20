---
type: episode-card
date: 2026-06-16
session: 412e32c5-05f9-4e2a-86c6-e1c21e464553
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/412e32c5-05f9-4e2a-86c6-e1c21e464553.jsonl
salience: architecture
status: active
subjects:
  - daemon-startup
  - relay-connection
  - daemon-state
supersedes: []
related_claims: []
source_lines:
  - 1-790
captured_at: 2026-06-18T00:39:20Z
---

# Episode: Daemon accept loop starts before relay connection

## Prior State

On cold start, the daemon bound its Unix socket then sequentially connected to relays (8+ seconds including NIP-42 AUTH warmup) before starting the accept loop. Clients that connected to the socket during this window hung waiting for a welcome that couldn't be served until relay warmup finished.

## Trigger

User reported tenex-edge tmux taking a very long time to start. Root-cause analysis revealed the socket was bound on server.rs:131 but the accept loop (line 208) didn't start until after Transport::connect completed — everything between those lines was dead time the client silently paid for.

## Decision

Restructured daemon startup to bind socket and start the accept loop immediately; relay connection moved to a background tokio task. DaemonState.transport and DaemonState.provider changed from Arc<Transport>/Arc<Kind1Nip29Provider> to Mutex<Option<Arc<...>>> with a relay_ready: Notify that fires once on connect. Store-only RPCs (who, tmux_*) now work instantly; relay-dependent operations await readiness via async fn transport()/provider().

## Consequences

- Store-only RPCs (who, tmux_*) render immediately from SQLite without waiting for relay connection
- Relay-dependent operations (publish, subscribe, session start) block via state.transport().await / state.provider().await until relay_ready fires
- ~20 call sites updated to use async accessor methods instead of direct field access
- DaemonState struct significantly restructured: transport/provider are now optional-mutexed, relay_ready Notify added
- reconcile_sessions and spawn_demux are called after set_relay() in the background task

## Open Tail

- Error path if relay connection fails after clients are already connected and awaiting readiness

## Evidence

- transcript lines 1-790

