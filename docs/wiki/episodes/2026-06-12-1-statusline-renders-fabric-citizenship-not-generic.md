---
type: episode-card
date: 2026-06-12
session: e42f09d7-5fb0-438b-a356-216870390540
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/e42f09d7-5fb0-438b-a356-216870390540.jsonl
salience: product
status: active
subjects:
  - tenex-edge-statusline
  - citizenship-line
supersedes: []
related_claims: []
source_lines:
  - 65-100
  - 200-330
captured_at: 2026-06-18T00:12:00Z
---

# Episode: Statusline renders fabric citizenship, not generic host data

## Prior State

No tenex-edge statusline existed; initial proposals were generic developer-tool bars (model name, git branch, context budget, dirty marker) with no connection to the project's core concepts.

## Trigger

User correction: 'that's not anchored enough on what this project is about… read the docs,' followed by user's explicit format specification with annotated segments: agent count (group members), session count (including idle), current activity, and inbox message (pending or recently consumed).

## Decision

The statusline is a one-line fabric awareness board: `claude@host [session-id] ⬡N ◉N ✎ activity ✉ inbox`. ⬡ = NIP-29 group member count (roster size, not just present peers), ◉ = live session count from heartbeat data, ✎ = self-reported session status (or `· idle`), ✉ = newest pending mention or `✉✓` recently-consumed (30s window). Membership warning (`⚠ not in group`) surfaces when agent is outside the project group. Quiet when all is well.

## Consequences

- ⬡ reads from group_members table (NIP-29 kind-39002 cache) — stable roster count, not transient presence
- ◉ reads from who-snapshot row count — sessions with non-expired heartbeats including idle
- Inbox uses peek_inbox (pending) plus new delivered_at column for 30s recently-consumed window
- ccstatusline multiplexes existing pc statusline (line 1) with tenex-edge line (line 2)
- Membership-warning state self-corrects as group cache refreshes, unlike injected context which LLMs can ignore

## Open Tail

- Remote machine at 157.180.102.242 runs old binary; sessions there won't render statusline until redeployed

## Evidence

- transcript lines 65-100
- transcript lines 200-330

