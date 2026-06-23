---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: product
status: superseded
subjects:
  - session-title
  - title-seed
  - distillation-timing
supersedes: []
related_claims: []
source_lines:
  - 618-622
  - 838-848
  - 894-994
captured_at: 2026-06-16T11:06:28Z
---

# Episode: Session title appears immediately and distills correctly

## Prior State

The title quick-seed read the transcript file via read_last_user_prompt(), but Claude Code hadn't flushed the just-submitted prompt to disk yet when turn_start fired, so turn 1 seeded empty; turn 2 read message 1. Separately, turn_first defaulted to 30s, but the obs loop only checks every 5s and most turns finish before 30s, so the LLM distiller was never scheduled (no error log because it never ran).

## Trigger

User reported: first message publishes no title (heartbeat empty), second message shows the first message as title, and LLM-based distillation never runs at all.

## Decision

(a) Thread the prompt text verbatim through the hook → turn_start RPC → persisted in a new last_user_prompt DB column; the runtime seed prefers this captured prompt over the lagging transcript read. (b) Lower turn_first default from 30s to 3s, below the 5s obs interval, so the first obs tick of any multi-second turn schedules the distill.

## Consequences

- Title appears on the very first message of a session, no longer one turn behind
- LLM distillation fires within the first ~5-10s of a turn instead of never
- New DB column last_user_prompt added to sessions table (migration: ALTER TABLE ADD COLUMN with default empty string)
- turn_start RPC params now include an optional prompt field; hook passes it through from user-prompt-submit

## Open Tail

*(none)*

## Evidence

- transcript lines 618-622
- transcript lines 838-848
- transcript lines 894-994

