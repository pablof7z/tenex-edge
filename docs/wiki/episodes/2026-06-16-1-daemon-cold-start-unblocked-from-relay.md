---
type: episode-card
date: 2026-06-16
session: 412e32c5-05f9-4e2a-86c6-e1c21e464553
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/412e32c5-05f9-4e2a-86c6-e1c21e464553.jsonl
salience: root-cause
status: active
subjects:
  - daemon-startup
  - daemon-state-transport-provider
  - relay-ready-gate
supersedes:
  - 2026-06-16-1-slow-cold-start-caused-by-socket
related_claims: []
source_lines:
  - 221-244
  - 689-706
  - 713-751
  - 755-780
  - 786-794
captured_at: 2026-06-16T10:08:33Z
---

# Episode: Daemon cold-start unblocked from relay warmup

## Prior State

On cold start, the daemon bound the Unix socket immediately, then blocked on Transport::connect() (relay connection + NIP-42 AUTH warmup, ~8+ seconds) before starting the accept loop. Clients connected to the socket instantly but then hung waiting for the daemon's welcome response until relay warmup completed. Store-only RPCs (who, tmux_*) that never touch relay were needlessly delayed.

## Trigger

User reported tenex-edge tmux taking a very long time to start. Root-cause diagnosis: socket bind on server.rs:131, but accept loop not spawned until line 208 — all relay connect + warmup time was dead time the client silently paid.

## Decision

DaemonState.transport and .provider changed from Arc<Transport>/Arc<Kind1Nip29Provider> to Mutex<Option<Arc<...>>> with a relay_ready: Notify gate. Accept loop now starts immediately after bind_socket() + store init. Relay connection runs in a spawned background task; once it completes, set_relay() atomically fills both fields and fires relay_ready.notify_waiters(). Store-only RPCs work instantly; relay-dependent RPCs call state.transport().await / state.provider().await which wake once the relay is up.

## Consequences

- TUI renders immediately from last-known SQLite state on cold start
- ~20 call sites updated to use the new async accessor methods
- New DaemonState helpers: async fn transport(), async fn provider(), fn provider_now() (non-blocking), fn set_relay()
- Background task also runs reconcile_sessions and spawn_demux after relay connects, preserving startup order for relay-dependent work
- relay_ready Notify is one-shot: once set, all waiters proceed without re-blocking

## Open Tail

*(none)*

## Evidence

- transcript lines 221-244
- transcript lines 689-706
- transcript lines 713-751
- transcript lines 755-780
- transcript lines 786-794

