---
type: episode-card
date: 2026-06-14
session: f0f28929-320e-4608-96bd-6f8ff7e0d602
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f0f28929-320e-4608-96bd-6f8ff7e0d602.jsonl
salience: product
status: superseded
subjects:
  - session-title-persistence
  - agent-active-flag
supersedes: []
related_claims: []
source_lines:
  - 50-51
  - 106-117
  - 139-148
  - 1105-1124
  - 1807-1831
captured_at: 2026-06-18T00:20:02Z
---

# Episode: Decouple session title from active/idle status

## Prior State

A single `status` string served double duty as both 'what the session is about' and 'is it working right now.' On turn-end the field was wiped to empty string → rendered as 'idle', erasing all context about what the agent was doing. No persistent title concept existed; the Activity struct was ephemeral and never stored in the database.

## Trigger

User directive: 'I want the title to be preserved a lot more, not just while the agent is working on it — the idle label should be independent of the agent's session title.' Further specification: title distilled on first turn, re-distilled on each new user message with current title fed back and a nudge to keep it unless work substantively changed; display = title + idle marker; lifetime = while session is alive.

## Decision

Split the single status field into two independent things: (1) a persistent title (Status.text) — distilled on first turn, re-distilled on each new user message via `distill::distill_title` with the current title fed back and a nudge-to-keep prompt, survives idle, cleared only on session exit; (2) an `active: bool` flag — the mid-turn indicator, decoupled from title, carried over the wire as an `['active','0'|'1']` NIP-38 tag, persisted in a new `active` column on session_status/agent_status tables. `is_idle()` became `!active` rather than 'empty text.'

## Consequences

- Display now shows the title always, with a dim '· idle' marker appended when not active; no title yet shows 'working' / 'idle'.
- Wire protocol (NIP-38) gained an `['active','0'|'1']` tag alongside existing `['expiration', ts]`; codec round-trip tests updated.
- DB migration adds `active INTEGER NOT NULL DEFAULT 0` column to both `session_status` and `agent_status` tables (additive, no data loss).
- New `distill::distill_title` function with nudge-to-keep system prompt; separate from `distill_activity`.
- `turn_repeat` default changed to 0 (opt-in in-turn safety re-distill) since each new user message already triggers re-distillation via rising-edge detection.
- WhoRow, TailEvent::Status, fabric delta block, and server backfill all carry the `active` field.
- Engine loop: title distilled on first observation (~30s into turn), re-distilled on rising edge of each new turn, preserved on falling edge (idle), cleared only on clean session exit.
- Another agent's compile-shims on master (which depended on these still-uncommitted signature changes) were reconciled via rebase; master was actually broken before this commit landed.

## Open Tail

- No live daemon smoke test was performed (the repo IS the running fabric); manual verification via TENEX_EDGE_DISTILL_CMD override suggested.
- Push to origin not yet done — local master only.

## Evidence

- transcript lines 50-51
- transcript lines 106-117
- transcript lines 139-148
- transcript lines 1105-1124
- transcript lines 1807-1831

