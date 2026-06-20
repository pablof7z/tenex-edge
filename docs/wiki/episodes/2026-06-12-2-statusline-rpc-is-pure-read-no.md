---
type: episode-card
date: 2026-06-12
session: e42f09d7-5fb0-438b-a356-216870390540
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/e42f09d7-5fb0-438b-a356-216870390540.jsonl
salience: architecture
status: active
subjects:
  - tenex-edge-daemon-rpc
  - statusline-architecture
supersedes: []
related_claims: []
source_lines:
  - 265-270
  - 714-735
captured_at: 2026-06-18T00:12:00Z
---

# Episode: Statusline RPC is pure-read, no-spawn, fail-open

## Prior State

No statusline daemon RPC existed; other verbs (turn_start, drain_inbox) write state. The documented multi-writer failure ('N per-session processes write to a single state.db is a confirmed failure mode') meant any new frequent caller risked re-introducing concurrent writers.

## Trigger

Claude Code re-runs the statusline command constantly (every 3s refresh); any write path would produce transient concurrent writers. A spawn-on-call would fight the daemon's idle-exit mechanism.

## Decision

The `statusline` RPC is pure-read (zero state.db writes — peeks inbox instead of draining, reads group_members instead of modifying). It uses a new `call_no_spawn` blocking client that never boots a daemon. It fails open: daemon unreachable → print nothing, exit 0. Protocol version not bumped (additive RPC only).

## Consequences

- New `call_no_spawn` in blocking.rs: connects to existing UDS or bails, never spawn-if-absent
- Inbox queries use `peek_inbox` + `list_recently_delivered` instead of `drain_inbox` — no state mutation
- New `delivered_at INTEGER NOT NULL DEFAULT 0` column on inbox table (migration: ALTER TABLE), enabling the recently-consumed 30s window without writes from the statusline path
- Daemon must be restarted (`pkill` + auto-respawn via `just install`) to expose the new RPC — old daemon returns error for unknown method

## Open Tail

*(none)*

## Evidence

- transcript lines 265-270
- transcript lines 714-735

