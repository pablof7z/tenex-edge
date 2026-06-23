---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: root-cause
status: active
subjects:
  - session-title-seed
  - spawn-session-atomicity
  - distill-timing
supersedes:
  - 2026-06-16-1-atomic-session-spawn-to-prevent-duplicate
related_claims: []
source_lines:
  - 372-409
  - 571-616
  - 618-841
  - 838-994
captured_at: 2026-06-16T11:19:04Z
---

# Episode: Session title publishing pipeline: three root-cause fixes

## Prior State

Three independent defects in the title pipeline: (1) spawn_session check-and-insert were separated by .await points, allowing two concurrent session_start RPCs to both spawn a runtime for the same session_id — producing zombie runtimes that flip-flop the 30315 event between two different quick-seeded titles; (2) turn_start received only the transcript path, but the runtime seed read read_last_user_prompt(transcript) before the harness flushed the new prompt to disk, so the first message showed an empty title and every subsequent message lagged one behind; (3) turn_first default was 30s but the obs loop ticks every 5s and most turns finish sooner, so the LLM distiller was never scheduled (no error log because it never ran).

## Trigger

User observed (a) the same session's kind:30315 relay event alternating between two titles every heartbeat, (b) first message producing no title then lagging one message behind, (c) LLM-distilled title never appearing.

## Decision

Three coordinated fixes: (1) spawn_session now atomically checks and reserves the session_id in state.sessions under one Mutex lock before any .await — second spawn for a live session_id returns early; (2) the prompt text is threaded through hooks.rs → turn.rs → rpc_turn_start and persisted in a new last_user_prompt sqlite column, so runtime.rs seeds the title from the captured prompt directly instead of the lagging transcript read; (3) turn_first default lowered from 30s to 3s so the first obs tick of any multi-second turn schedules the distiller.

## Consequences

- Only one runtime per session_id can exist — duplicate session_start RPCs are idempotent
- First user message immediately seeds a title; no more one-turn lag
- LLM distillation fires within ~5s of turn start instead of never
- Existing zombie runtimes in the running daemon must be cleared by restart

## Open Tail

- opencode harness still mints a fresh te-… session_id on every start, creating duplicate sessions with different d-tags — separate fix needed
- killed stale sessions leave orphaned kind:30315 events on the relay (never expired) with no deletion mechanism

## Evidence

- transcript lines 372-409
- transcript lines 571-616
- transcript lines 618-841
- transcript lines 838-994

