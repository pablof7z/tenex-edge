---
type: episode-card
date: 2026-06-14
session: 215d979a-a054-4e2b-b349-851e0d874d6d
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/215d979a-a054-4e2b-b349-851e0d874d6d.jsonl
salience: root-cause
status: superseded
subjects:
  - engine-distillation
  - session-title
  - runtime-loop
supersedes: []
related_claims: []
source_lines:
  - 137-1270
captured_at: 2026-06-18T00:30:05Z
---

# Episode: Session distillation engine: immediate title seeding, async with timeout, retry on failure

## Prior State

Distillation was awaited inline inside `tokio::select!`, blocking all engine work (heartbeats, presence) for the full duration of the OpenRouter API call (observed 6+ minutes). `last_distill = now` was set unconditionally even on `None` (failure), consuming the single retry slot per turn since `turn_repeat=0`. No title existed until the LLM call succeeded, leaving sessions displayed as just 'working' with no descriptive label.

## Trigger

User observed that their current session had no title in `tenex-edge tmux` for 15+ minutes. Root-cause analysis revealed: (1) `complete_via_rig` blocks the engine indefinitely, (2) `last_distill` set on failure prevents retry.

## Decision

Three fixes implemented: (1) On rising edge (new turn detected) with `cur_title.is_none()`, immediately read the last user prompt from transcript via new `read_last_user_prompt`, titleize+truncate to 60 chars, and publish as the title before any LLM call. (2) Distillation runs in a spawned `JoinHandle` with `tokio::time::timeout(20s, ...)` — engine polls `is_finished()` non-blockingly, staying fully responsive. (3) `last_distill` only written on successful `Some(labels)` result; separate `last_distill_attempt` timestamp gates retries so a failed distillation retries after another `turn_first` window.

## Consequences

- Sessions get a readable title within seconds of the first user prompt, regardless of LLM latency or failures
- Engine loop never blocks on distillation — heartbeats and presence continue during slow API calls
- Failed distillation calls (timeout, API error) are retried rather than silently consumed
- Falling edge (idle transition) resets distill_task, last_distill_attempt, and distill_task_turn to avoid stale results
- transcript.rs gained read_last_user_prompt — parses the tail of the JSONL to extract the last user text message

## Open Tail

- The `turn_repeat` interval is 0 (disabled) in production — re-distillation within a turn never triggers, so only the initial distillation and immediate title seeding provide titles. If within-turn re-distillation is desired later, the retry logic is already wired.

## Evidence

- transcript lines 137-1270

