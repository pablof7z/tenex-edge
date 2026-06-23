---
type: episode-card
date: 2026-06-12
session: e42f09d7-5fb0-438b-a356-216870390540
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/e42f09d7-5fb0-438b-a356-216870390540.jsonl
salience: product
status: active
subjects:
  - tenex-edge-statusline
  - inbox-delivered-at
supersedes:
  - 2026-06-12-1-statusline-re-anchored-from-generic-git
  - 2026-06-12-1-statusline-redesigned-as-citizenship-awareness-line
related_claims: []
source_lines:
  - 65-278
  - 714-987
captured_at: 2026-06-12T19:40:00Z
---

# Episode: Statusline as citizenship line, not generic model bar

## Prior State

No statusline existed; initial proposals were generic (model, branch, git dirty count) and not grounded in tenex-edge's fabric concepts.

## Trigger

User rejected generic proposals ('that's not anchored enough on what this project is about… read the docs') and then specified the exact format: `claude@host [session-id] #agent-symbol #sessions-symbol [current-activity] [inbox-message]` with annotations mapping each segment to fabric concepts (group membership count, active sessions, self-reported status, recently-consumed or pending inbox).

## Decision

The statusline renders a 'citizenship line' — `claude@host [session-id] ⬡N ◉N ✎ activity ✉ inbox` — showing identity, NIP-29 group member count (⬡), live session count from who-snapshot (◉), self-reported session status (✎), and inbox envelope (pending or recently-consumed within 30s as ✉✓). Membership-warning state (`⚠ not in group project`) shown when agent is not a group member. The `delivered_at` column was added to the inbox table to support the 30-second recently-consumed window.

## Consequences

- New `delivered_at INTEGER NOT NULL DEFAULT 0` column added to inbox schema with ALTER TABLE migration; `drain_inbox` now timestamps deliveries
- New `list_recently_delivered` query on Store for the 30s window
- New `rpc_statusline` daemon RPC: pure-read (no state.db writes, avoiding the documented multi-writer failure mode), composed from existing `load_who_snapshot`, `count_group_members`, `peek_inbox`, and `list_recently_delivered`
- New `call_no_spawn` blocking client: statusline CLI must never boot a daemon (would fight idle-exit since the harness calls it every 3s)
- CLI fails open: daemon unreachable → print nothing, exit 0
- ccstatusline multiplexes existing `pc statusline` (line 1) with `tenex-edge statusline` (line 2)
- Remote machine at 157.180.102.242 needs redeploy to expose the new RPC

## Open Tail

- Remote machine not yet redeployed — sessions there will not render a statusline until binary is updated
- Code changes uncommitted (other agent sessions active in the worktree)

## Evidence

- transcript lines 65-278
- transcript lines 714-987

