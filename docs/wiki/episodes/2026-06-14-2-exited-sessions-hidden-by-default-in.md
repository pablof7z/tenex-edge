---
type: episode-card
date: 2026-06-14
session: 656e1e6b-2569-42da-8844-768a5e74788e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/656e1e6b-2569-42da-8844-768a5e74788e.jsonl
salience: product
status: superseded
subjects:
  - tenex-edge-tmux-tui
  - exited-sessions-visibility
supersedes: []
related_claims: []
source_lines:
  - 1-7
  - 319-320
  - 372-374
captured_at: 2026-06-18T00:27:48Z
---

# Episode: Exited sessions hidden by default in TUI

## Prior State

Exited/resumable sessions were always visible in the TUI, mixed in with live sessions

## Trigger

User explicitly requested: 'exited sessions should not be shown by default, I should need to press a key to see them'

## Decision

Exited sessions are hidden by default; press 'e' to toggle visibility. Help line updates to show [e] hide exited when they're visible, and [e] show exited when hidden.

## Consequences

- The old 'Resumable' section is no longer always rendered
- Users must opt-in to see exited sessions, reducing visual noise for common case

## Open Tail

*(none)*

## Evidence

- transcript lines 1-7
- transcript lines 319-320
- transcript lines 372-374

