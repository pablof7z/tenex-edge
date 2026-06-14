---
type: episode-card
date: 2026-06-14
session: 656e1e6b-2569-42da-8844-768a5e74788e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/656e1e6b-2569-42da-8844-768a5e74788e.jsonl
salience: product
status: superseded
subjects:
  - tenex-edge
  - tmux-tui
  - exited-sessions-visibility
supersedes: []
related_claims: []
source_lines:
  - 1-7
captured_at: 2026-06-14T18:52:26Z
---

# Episode: Exited sessions hidden by default in tmux TUI

## Prior State

Exited/resumable sessions were shown by default alongside live sessions in the TUI.

## Trigger

User directive on line 3: 'exited sessions should not be shown by default, I should need to press a key to see them.'

## Decision

Exited sessions are now hidden by default; pressing 'e' toggles their visibility. The help line dynamically updates to show '[e] hide exited' when visible and '[e] show exited' when hidden.

## Consequences

- Default TUI view is cleaner, showing only live sessions and agents
- Users must explicitly opt in to see exited/resumable sessions

## Open Tail

*(none)*

## Evidence

- transcript lines 1-7

