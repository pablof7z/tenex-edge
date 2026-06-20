---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: product
status: active
subjects:
  - status-delta-self-exclusion
  - fabric-context
supersedes: []
related_claims: []
source_lines:
  - 2406-2462
captured_at: 2026-06-18T00:45:05Z
---

# Episode: Self-exclude viewer's own session from turn-start deltas

## Prior State

build_status_delta returned all changed sessions since the previous turn, including the viewer's own session. A session would see its own 'just started working' change echoed back in its injected context.

## Trigger

Identified as a credible medium-severity issue during the bug audit: turn-start deltas not self-excluded → session sees its own change.

## Decision

push_turn_fabric_block now accepts a self_session parameter (the viewer's canonical session id). build_status_delta filters out the viewer's own session from the delta, so an agent never sees its own state change in its injected fabric context.

## Consequences

- First turn still shows the full roster (no filtering on first turn)
- Subsequent turns see only peer changes — never their own session's transition

## Open Tail

*(none)*

## Evidence

- transcript lines 2406-2462

