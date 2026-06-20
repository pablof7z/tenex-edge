---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: architecture
status: active
subjects:
  - session-state-aggregate
  - canonical-id-resolution
  - turn-transition-ownership
supersedes: []
related_claims: []
source_lines:
  - 2058-2326
captured_at: 2026-06-18T00:45:05Z
---

# Episode: Single-owner turn transitions via canonical session ID

## Prior State

Turn transitions (start_turn/end_turn) were called by both the RPC layer and the runtime observer, both using the raw harness session ID (e.g. Claude's native id). The runtime polled turn_state by the canonical (minted) id, so it never saw RPC-written transitions. This meant sessions never went busy for Claude/Codex, turn_id double-incremented, and the aggregate was inert for the primary agents.

## Trigger

Code review identified that rpc_turn_start/rpc_turn_end/rpc_session_end operated on p.session (harness id) while runtime polled by p.session_id (canonical), and both RPC and runtime called start_turn/end_turn causing double ownership.

## Decision

RPC is now the sole owner of start_turn/end_turn/end_session transitions. Runtime only observes turn_state (reads, never writes transitions). All RPC handlers (turn_start, turn_end, session_end) resolve harness→canonical id via new canonical_session_id() before any state mutation. cancel_session and end_session now use rec.session_id (canonical) instead of raw p.session.

## Consequences

- Claude/Codex sessions now correctly transition busy→idle around turns
- turn_id increments once per turn, not twice
- Runtime no longer mutates turn state — it only reads turn_state and applies distill results
- New regression test turn_lifecycle_by_harness_alias_drives_canonical_row validates the full hook path by harness alias

## Open Tail

- Upgrade path intentionally does not backfill old sessions table → session_state; sessions re-register on their next hook (cleans zombie pile)

## Evidence

- transcript lines 2058-2326

