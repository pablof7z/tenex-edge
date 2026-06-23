---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: root-cause
status: active
subjects:
  - distill-timing
  - turn-first
  - title-generation
supersedes:
  - 2026-06-16-3-lower-turn-first-default-so-distillation
related_claims: []
source_lines:
  - 952-976
captured_at: 2026-06-16T11:28:20Z
---

# Episode: Lower turn_first from 30s to 3s so the distiller actually fires

## Prior State

turn_first default was 30 seconds. The observation loop ticks every 5s and checks whether a distill is due. Most Claude turns finish before 30s, so the distill task was never scheduled — the falling edge dropped it. No error log because it never ran.

## Trigger

Finding that the distiller never ran (empty error log for the session), traced to the 30s turn_first gate being longer than typical turn duration.

## Decision

Changed turn_first default from 30s to 3s, below the 5s obs interval, so the first obs tick of any multi-second turn schedules the distill.

## Consequences

- LLM-based titles now actually get generated during turns
- May cause more distill API calls for very short turns; mitigated by turn_repeat remaining 0

## Open Tail

*(none)*

## Evidence

- transcript lines 952-976

