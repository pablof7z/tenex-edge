---
type: episode-card
date: 2026-06-14
session: 4ba07cd0-c4df-4e63-ae13-90c20c46f6ce
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/4ba07cd0-c4df-4e63-ae13-90c20c46f6ce.jsonl
salience: product
status: active
subjects:
  - tenex-edge-who-host-display
  - render-who-row
  - render-who-plain
supersedes:
  - 2026-06-12-1-who-command-show-hostname-instead-of
related_claims: []
source_lines:
  - 1-599
captured_at: 2026-06-18T00:24:07Z
---

# Episode: who always shows host, including same-machine agents

## Prior State

The who command suppressed host for same-machine agents (§8e decision to reduce clutter); only genuine cross-machine peers got a `(remote)` tag.

## Trigger

User explicitly requested: 'make tenex-edge who include the host where the agent is running (even if its the same host)'

## Decision

Every agent row now displays its host: same-machine agents show `(laptop)`, cross-machine agents show `(tenex kind2, remote)` — hostname plus a `, remote` flag to keep them distinguishable. Both renderers updated (rich CLI and plain turn-fabric block).

## Consequences

- Both render_who_row (rich CLI) and render_who_plain (turn-fabric context) now include host for all agents
- All 8 who_tests updated to assert host presence in output
- Discovered src/cli/who.rs and its submodules are dead code — never declared with `mod who;`, so live implementation is inline in cli.rs (latent footgun from commit bd91c352)
- Had to add missing `attachable: false` field to two test literals that a concurrent session added to WhoRow but left incomplete

## Open Tail

- The dead-code src/cli/who.rs module tree should be cleaned up to avoid future confusion over 'the correct WhoSnapshot'

## Evidence

- transcript lines 1-599

