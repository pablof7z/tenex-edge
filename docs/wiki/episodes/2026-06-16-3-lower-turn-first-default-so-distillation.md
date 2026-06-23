---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: product
status: superseded
subjects:
  - title-distillation
  - turn-first-timing
  - obs-loop
supersedes: []
related_claims: []
source_lines:
  - 620-995
captured_at: 2026-06-16T11:11:54Z
---

# Episode: Lower turn_first default so distillation actually fires within a turn

## Prior State

turn_first default was 30 seconds. The observation loop (obs_interval) ticks every 5 seconds and checks 'is a distill due?'. Most turns finish before 30 seconds, so the distiller was never scheduled. No error log was produced because it simply never ran. The LLM-based title refinement feature was effectively dead code.

## Trigger

User reported: 'it NEVER actually creates the llm-based title it should have generated'. Investigation confirmed no distill error logs for the session and the 30s default exceeded typical turn duration.

## Decision

Changed turn_first default from 30s to 3s, well below the 5s obs_interval, so the first obs tick of any multi-second turn schedules the distillation.

## Consequences

- Distiller now fires early in the turn, producing refined LLM-generated titles
- turn_repeat remains 0 (disabled), meaning re-distillation within a turn is opt-in via env var

## Open Tail

*(none)*

## Evidence

- transcript lines 620-995

