---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: root-cause
status: active
subjects:
  - session-lifecycle
  - spawn-session
  - daemon-state
supersedes: []
related_claims: []
source_lines:
  - 550-608
captured_at: 2026-06-16T11:28:20Z
---

# Episode: Atomic spawn_session to prevent duplicate runtimes

## Prior State

The 'already running?' check and the map insert in spawn_session were separated by .await points (open_project, ensure_subscription). Two session_start RPCs for the same session_id could both pass the check, both spawn runtimes, and the second insert would evict the first from the sessions map — leaving an orphaned runtime that heartbeats forever as a zombie, causing title flip-flopping on the relay.

## Trigger

Root-cause finding: the DB showed two session_ids for the same pid (te-18b984c3975a5698-29127 and te-18b9848dc6bc7708-29127), and relay events for session e6012034 alternated between two titles with staggered created_at timestamps — proving two independent runtimes were heartbeating the same d-tag.

## Decision

spawn_session now does an atomic check-and-reserve into state.sessions under one lock before any await. Second spawn for a live session_id returns early. Reservation rolls back if subscription setup fails.

## Consequences

- Prevents new zombie runtimes from being spawned
- Existing orphaned runtimes in the running daemon must be cleared by restart
- Belt-and-suspenders once the session aggregate architecture lands

## Open Tail

- The opencode gen_session_id()-on-every-start bug still mints fresh te-* ids per prompt, creating duplicate sessions with different d-tags

## Evidence

- transcript lines 550-608

