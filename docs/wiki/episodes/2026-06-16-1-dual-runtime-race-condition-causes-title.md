---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: root-cause
status: superseded
subjects:
  - spawn-session-race
  - session-title-publishing
  - kind30315-event
supersedes: []
related_claims: []
source_lines:
  - 372-616
captured_at: 2026-06-16T10:59:44Z
---

# Episode: Dual-runtime race condition causes title flip-flop on 30315 events

## Prior State

spawn_session's 'already running?' guard (checking sessions map) and the actual map insert were separated by two .await points (open_project, ensure_subscription). Two concurrent session_start RPCs for the same session_id could both pass the guard, both spawn an engine runtime, and the second insert would evict the first runtime from the sessions map — making it un-cancellable.

## Trigger

User reported that a single session's kind:30315 relay event alternates between two titles (message1 ↔ message2) on every heartbeat. Relay dump confirmed same d-tag, same pubkey, alternating created_at timestamps with different title values. DB query showed two generated session_ids with the same pid (te-…-29127), confirming session_start fired twice.

## Decision

Made spawn_session atomically check-and-reserve into state.sessions under one lock before any .await. A second spawn for a live session_id now returns early. Reservation is rolled back if subscription setup fails, so no stale entry leaks.

## Consequences

- New duplicate spawns are prevented — second session_start for an already-live session_id is a no-op
- Existing zombie runtimes in the currently-running daemon are unaffected; daemon must be rebuilt and restarted to clear them
- The opencode harness bug (gen_session_id mints a brand-new te-… id per session_start call, creating distinct d-tags for the same conversation) remains unfixed

## Open Tail

- First message to claude-code publishes empty title (heartbeat but no seed); title only appears after second message — lagging seed path not yet traced
- LLM-based title distillation (distill_session) never actually fires or produces output for the affected session — cause unknown
- Opencode's gen_session_id producing different ids per session_start call creates duplicate sessions with different d-tags

## Evidence

- transcript lines 372-616

