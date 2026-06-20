---
type: episode-card
date: 2026-06-16
session: a7c75cc2-efc0-47db-aa7d-9332d6c63310
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a7c75cc2-efc0-47db-aa7d-9332d6c63310.jsonl
salience: product
status: active
subjects:
  - tmux-attach-fallback
  - pending-attach-struct
supersedes: []
related_claims: []
source_lines:
  - 1-8
  - 94-137
captured_at: 2026-06-18T00:42:28Z
---

# Episode: Stale tmux pane attach now falls back to transparent resume

## Prior State

When a tmux pane was reported as attachable but had since vanished (stale pane id), the TUI surfaced a raw dead-end error: "Attach failed: pane %110 not found" or "Session pane not found." The user had no way to proceed.

## Trigger

User directive: "this error should never exist — if the tmux pane is not attachable then we just resume the session as if it weren't attached to a tmux... that's it."

## Decision

Attach is now best-effort with automatic resume fallback. PendingAttach struct expanded to carry both pane_id and resume_sid. Three paths changed: (1) blocking attach site — if attach_pane_blocking fails, transparently falls back to resume_in_tui and re-attaches to the fresh pane; (2) Enter on an attachable session with no live pane — resumes directly instead of showing "Session pane not found"; (3) resume key path — same fallback semantics. Error only surfaces if resume itself also fails.

## Consequences

- Users never see stale-pane dead-end errors; sessions are always recoverable
- Spawned sessions have no resume fallback (resume_sid = None), which is correct since there is no prior session to resume
- PendingAttach is no longer a simple Option<String> — all four pending_attach assignment sites must supply the resume_sid

## Open Tail

*(none)*

## Evidence

- transcript lines 1-8
- transcript lines 94-137

