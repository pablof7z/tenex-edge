---
type: episode-card
date: 2026-06-14
session: 84aaaa96-2082-42b1-b9af-83f9c8f90f67
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/84aaaa96-2082-42b1-b9af-83f9c8f90f67.jsonl
salience: product
status: active
subjects:
  - session-labels
  - distill-prompt
  - who-display
  - nip29-status-event
supersedes: []
related_claims: []
source_lines:
  - 54-93
  - 95-134
  - 385-434
  - 643-698
  - 1399-1458
captured_at: 2026-06-14T15:19:23Z
---

# Episode: Combined session-label distillation replaces narrow title-only prompt

## Prior State

Two separate prompts existed: TITLE_SYSTEM_PROMPT (which used 'present tense, intent not mechanics' — the same framing as the live-activity prompt) and RIG_SYSTEM_PROMPT for the activity line. The title prompt produced narrow, step-level descriptions like 'Finding github issue repository' instead of goal-level titles like 'Fix GitHub issue 1' because it had no outlet for current-action content, so mechanics leaked into the title. The runtime had already collapsed the live-activity distiller (only distill_title was called), but the title prompt still inherited activity-line framing. The `who` display showed only title text + idle/busy boolean.

## Trigger

User observed that generated session titles were too narrow — e.g., 'Finding github issue repository' when the user asked 'Fix github issue 1'. The root cause was identified: the title prompt was a clone of the activity-line prompt with 'present tense' framing, giving the model nowhere to put current-action content, so it leaked into the title.

## Decision

Replaced the two separate prompts (RIG_SYSTEM_PROMPT + TITLE_SYSTEM_PROMPT) with a single combined SESSION_SYSTEM_PROMPT that outputs two labeled lines — TITLE (stable session objective) and NOW (ephemeral current activity) — in one LLM call. The title prompt now explicitly asks for the objective/goal ('prefer the user's stated request'), not present-tense action, and the inline example directly contrasts the failure case. The `activity` field was threaded through the entire stack: Status struct, NIP-29 codec (optional `['activity', …]` tag on kind:30315), SQLite schema (ALTER TABLE additions), runtime (single distill_session call sets both; activity cleared on idle/new turn; title persists), and `who` rendering (`Fix GitHub issue 1 — reading the issue tracker · busy`, collapsing to `Fix GitHub issue 1 · idle`).

## Consequences

- One LLM call per distill cycle (same count as before, slightly more output tokens) instead of re-introducing a second call.
- The altitude bug is structurally fixed: the model has an explicit NOW: slot for step-mechanics, so they no longer leak into the title.
- Activity is ephemeral (cleared on idle and each new turn, not recovered across daemon restart); title persists.
- The activity tag on the NIP-29 wire format is additive/optional — older peers that don't know activity ignore it and still read the title from content, so no version bump needed.
- The old Activity domain event variant and RIG_SYSTEM_PROMPT/summarize_via_rig remain in code but are dead paths (runtime no longer produces Activity events).
- The turn-delta feed (list_status_changes_since) still shows title-only; activity could be threaded through later.

## Open Tail

- User's final screenshot shows no title generated at all for an opencode agent — may indicate the distill command or rig provider is not configured for that agent, or a runtime wiring issue.
- The old Activity domain event and summarize_via_rig are now dead code and could be removed in a cleanup pass.

## Evidence

- transcript lines 54-93
- transcript lines 95-134
- transcript lines 385-434
- transcript lines 643-698
- transcript lines 1399-1458

