---
type: episode-card
date: 2026-06-14
session: 9f7f245f-0fad-4211-a86b-95ea3cbb532e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9f7f245f-0fad-4211-a86b-95ea3cbb532e.jsonl
salience: root-cause
status: active
subjects:
  - tmux-attach
  - view-session
  - reaping-race
supersedes: []
related_claims: []
source_lines:
  - 1561-1565
  - 1721-1730
captured_at: 2026-06-14T15:19:34Z
---

# Episode: TUI Attach View Session Reaping Race

## Prior State

`ensure_view_session` created per-client tmux view sessions with `destroy-unattached on`, intended to auto-clean views when no client was attached.

## Trigger

Attach failed with 'can't find session: tenex-view-XXXXX' — the view session was reaped instantly because it was created in a detached state (zero clients), so `destroy-unattached` destroyed it before the subsequent attach/switch-client could reach it.

## Decision

Replaced `destroy-unattached on` with a `client-detached` hook on the view session, so it survives until a real client attaches and is cleaned up only after that client detaches.

## Consequences

- Attach from outside tmux now works reliably without race conditions
- Silenced the misleading `has-session` stderr that printed 'can't find session' for expected misses (the existence check that's supposed to fail when the view doesn't exist yet)
- Also fixed latent edge case: empty TMUX env var was treated as 'in tmux', now correctly treated as not-in-tmux

## Open Tail

*(none)*

## Evidence

- transcript lines 1561-1565
- transcript lines 1721-1730

