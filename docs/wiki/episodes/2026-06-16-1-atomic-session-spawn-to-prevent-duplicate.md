---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: root-cause
status: superseded
subjects:
  - session-spawn
  - daemon-lifecycle
  - status-event-30315
supersedes:
  - 2026-06-16-1-atomic-session-spawn-prevents-zombie-runtimes
related_claims: []
source_lines:
  - 374-616
captured_at: 2026-06-16T11:11:54Z
---

# Episode: Atomic session spawn to prevent duplicate runtime zombies

## Prior State

spawn_session checked session_id existence at server.rs:636 but inserted into the sessions map at server.rs:2791, with two .await points (open_project, ensure_subscription) in between. Two near-simultaneous session_start RPCs for the same session_id both passed the guard, both spawned engine runtimes, and the second insert evicted the first from the map — making it un-cancellable. Each runtime held its own cur_title and heartbeat, producing flip-flopping kind:30315 replaceable events on the relay.

## Trigger

User observed the 30315 event alternating between two titles for the same session_id and d-tag. DB query showed two te- prefixed session ids sharing the same watch_pid (29127), confirming session_start fired twice. Relay dump showed the same d-tag with alternating titles at ~5s intervals.

## Decision

spawn_session now atomically checks and reserves the session_id in state.sessions under a single Mutex lock, before any .await. A second spawn for a live session_id returns early. The reservation is rolled back if subscription setup fails.

## Consequences

- Prevents orphaned runtimes that can never be cancelled
- Stops the relay flip-flop where two writers alternate on the same replaceable d-tag
- Existing zombie runtimes in the running daemon must be cleared by restart

## Open Tail

- The related opencode bug (gen_session_id mints a fresh te- id on every prompt → duplicate sessions with different d-tags) is still unfixed

## Evidence

- transcript lines 374-616

