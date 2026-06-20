---
type: episode-card
date: 2026-06-14
session: 9f7f245f-0fad-4211-a86b-95ea3cbb532e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9f7f245f-0fad-4211-a86b-95ea3cbb532e.jsonl
salience: product
status: active
subjects:
  - tui-attach
  - tmux-session-model
supersedes: []
related_claims: []
source_lines:
  - 2083-2158
  - 2203-2211
captured_at: 2026-06-18T00:26:16Z
---

# Episode: TUI inline attach with return-to-list replaces exit-and-exec

## Prior State

Attaching from the TUI replaced the process (`exec tmux attach-session`), so the TUI was destroyed and users had to re-launch `tenex-edge tmux` after detaching.

## Trigger

Implicit in the per-agent session redesign — with each agent in its own session, the natural interaction is to temporarily attach and return to the list, not exit permanently.

## Decision

The TUI now suspends (leaves alt-screen/raw-mode), runs `tmux attach-session` as a blocking child process (with `$TMUX` stripped so it nests even when launched inside tmux), and resumes the TUI loop when the user detaches (`Ctrl-b d`).

## Consequences

- `tenex-edge tmux` keeps running underneath; detaching drops back into the session list
- Nested tmux prefix (`Ctrl-b` twice) needed when running inside an existing tmux session
- Eliminated the `TuiExit` enum — the loop no longer exits on attach

## Open Tail

- Nested tmux prefix ergonomics need real-terminal validation

## Evidence

- transcript lines 2083-2158
- transcript lines 2203-2211

