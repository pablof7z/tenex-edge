---
type: episode-card
date: 2026-06-14
session: 84aaaa96-2082-42b1-b9af-83f9c8f90f67
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/84aaaa96-2082-42b1-b9af-83f9c8f90f67.jsonl
salience: product
status: active
subjects:
  - session-title-distillation
  - who-display
  - status-activity-model
supersedes:
  - 2026-06-14-1-decouple-session-title-from-active-idle
related_claims: []
source_lines:
  - 54-56
  - 95-98
  - 110-134
  - 297-297
  - 385-434
  - 436-438
  - 1367-1397
  - 1460-1464
captured_at: 2026-06-18T00:23:15Z
---

# Episode: Session distillation restructured: single-prompt TITLE+NOW replaces dual prompts

## Prior State

Session titles used TITLE_SYSTEM_PROMPT (which inherited 'present tense, intent not mechanics' from the activity-line prompt) and activity used a separate RIG_SYSTEM_PROMPT. The title prompt produced narrow, step-level descriptions (e.g. 'Finding github issue repository' when the user asked to 'Fix github issue 1') because the model had nowhere to put 'now' content, so it leaked into the title. A prior refactor had already disconnected the activity-line distiller from the runtime loop — only distill_title was called, so the live activity was dead code.

## Trigger

User observed titles were too narrow/current-action-level instead of objective-level. Example: 'Fix github issue 1' became 'Finding github issue repository.' User then requested that both title and current activity be generated together in one prompt, after initially saying to drop activity entirely.

## Decision

Replaced the two separate prompts with a single SESSION_SYSTEM_PROMPT that outputs two labeled lines (TITLE: <objective> and NOW: <current step>). Title reframed as 'the objective the agent was asked to accomplish' (not present-tense action); NOW absorbs the step-mechanics pressure that previously leaked into titles. One LLM call produces both fields. Activity field was plumbed through the full stack: Status struct, kind:30315 codec (as optional ['activity'] tag), state schema (new column with ALTER TABLE migration), runtime loop (cleared on idle/new turn), and WhoRow renderer (shows 'Title — activity · busy', collapsing to 'Title · idle').

## Consequences

- The title altitude bug is fixed at root: giving the model a NOW slot removes pressure to stuff step-mechanics into the title.
- The prior RIG_SYSTEM_PROMPT and distill_activity function are replaced by distill_session returning SessionLabels{title, activity}.
- Wire format is backward-compatible: the activity tag is optional/additive, so older peers ignore it.
- Cost unchanged: still one LLM call per distill, just a few more output tokens.
- Live activity is ephemeral — cleared on idle and not recovered across daemon restart, while title persists.
- The turn-delta feed (list_status_changes_since) still shows title-only; activity is not yet threaded there.

## Open Tail

- User's final screenshot showed no title being generated at all for an opencode agent session — root cause unknown, may be a distiller invocation issue or a cold-start problem with the new prompt.
- Turn-delta status changes (↻ lines) don't yet show the activity field.

## Evidence

- transcript lines 54-56
- transcript lines 95-98
- transcript lines 110-134
- transcript lines 297-297
- transcript lines 385-434
- transcript lines 436-438
- transcript lines 1367-1397
- transcript lines 1460-1464

