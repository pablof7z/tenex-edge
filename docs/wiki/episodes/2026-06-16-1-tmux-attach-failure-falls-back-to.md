---
type: episode-card
date: 2026-06-16
session: a7c75cc2-efc0-47db-aa7d-9332d6c63310
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a7c75cc2-efc0-47db-aa7d-9332d6c63310.jsonl
salience: product
status: active
subjects:
  - tmux-attach-fallback
  - session-resume
supersedes: []
related_claims: []
source_lines:
  - 1-137
captured_at: 2026-06-16T10:45:43Z
---

# Episode: Tmux attach failure falls back to resume instead of surfacing error

## Prior State

When a daemon-reported attachable tmux pane had vanished, the TUI surfaced a dead-end error like 'Attach failed: pane %110 not found in any tmux session', blocking the user from resuming the session.

## Trigger

User directive: 'this error should never exist — if the tmux pane is not attachable then we just resume the session as if it weren't attached to a tmux'; root cause was that the daemon could report a session as attachable with a stale pane id, and attach-session would then fail with no fallback.

## Decision

Attach is now best-effort: if attaching to the pane fails, the TUI transparently resumes the session via the daemon and attaches to the fresh pane — exactly as if it had never been in tmux. PendingAttach struct extended to carry a resume_sid fallback; all four pending_attach sites (live attach, spawn, Enter-resume, r-resume) updated.

## Consequences

- PendingAttach struct now carries both a pane id and a resume_sid; freshly spawned panes get None for the fallback.
- The 'pane not found' error class can never reach the user — only a resume failure itself would surface.
- Enter on a flagged-attachable session with no live pane now resumes directly instead of showing 'Session pane not found.'

## Open Tail

*(none)*

## Evidence

- transcript lines 1-137

