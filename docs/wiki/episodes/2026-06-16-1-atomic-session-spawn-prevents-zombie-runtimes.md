---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: root-cause
status: superseded
subjects:
  - session-lifecycle
  - spawn-session
  - zombie-runtime
supersedes:
  - 2026-06-16-1-dual-runtime-race-condition-causes-title
related_claims: []
source_lines:
  - 374-409
  - 550-593
  - 604-616
captured_at: 2026-06-16T11:06:28Z
---

# Episode: Atomic session spawn prevents zombie runtimes

## Prior State

spawn_session checked whether a session_id was already live, then performed .await operations (open_project, ensure_subscription) before inserting into the sessions map — a TOCTOU race allowing two concurrent session_start RPCs for the same id to both pass the guard and spawn separate runtimes.

## Trigger

User observed a single session's kind:30315 event flip-flopping between two different titles on alternating heartbeats. DB query showed duplicate sessions (te-18b984c3975a5698-29127 and te-18b9848dc6bc7708-29127) sharing the same pid. Relay dump confirmed two writers publishing to the same d-tag with staggered created_at timestamps.

## Decision

spawn_session now atomically checks and reserves the session_id in state.sessions under a single Mutex lock before any .await point. A second spawn for an already-live session_id returns early. The reservation is rolled back if subscription setup fails.

## Consequences

- No new zombie runtimes can form — the second RPC simply returns instead of spawning a competing engine task
- Existing zombie runtimes in the running daemon are NOT automatically killed — a daemon restart is required to clear them
- The orphaned-runtime problem (evicted from map → un-cancellable → heartbeats forever) is structurally impossible going forward

## Open Tail

- The opencode harness also mints a fresh gen_session_id() on every session_start call, creating different d-tags for the same logical conversation — that is a separate bug not yet fixed

## Evidence

- transcript lines 374-409
- transcript lines 550-593
- transcript lines 604-616

