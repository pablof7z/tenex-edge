---
type: episode-card
date: 2026-06-14
session: bb7ee4ef-16bf-41b9-8e75-ed6b23f0f3a4
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/bb7ee4ef-16bf-41b9-8e75-ed6b23f0f3a4.jsonl
salience: product
status: active
subjects:
  - tui-attach
  - tmux-session-display
supersedes: []
related_claims: []
source_lines:
  - 1954-1956
  - 1992-2094
  - 2096-2121
  - 2157-2160
  - 2162-2183
captured_at: 2026-06-18T00:18:47Z
---

# Episode: Non-attachable sessions: TUI marks and blocks unattachable sessions

## Prior State

All live sessions in the TUI appeared equally selectable; pressing Enter on any session attempted tmux attach regardless of whether the session had a registered tmux endpoint, producing 'Not in tmux or select-pane failed' for non-tmux sessions; attach used select-pane which only works for panes in the current tmux window

## Trigger

User reported that selecting colored (remote/non-tmux) sessions and pressing Enter produced 'Not in tmux or select-pane failed' error; second report confirmed the fix for unattachable marking still failed for attachable sessions outside tmux

## Decision

Added attachable: bool to WhoRow, populated by checking whether a tmux endpoint is registered for the session's session_id; non-attachable sessions render dimmed with '[no tmux]' tag and Enter is blocked with a message; inside tmux, attach uses switch-client -t (works cross-window) instead of select-pane -t (same-window only); outside tmux, attach execs tmux attach-session -t session:window; spawn auto-switches to the new pane on success

## Consequences

- Remote sessions and sessions without tmux panes are visually distinct (dimmed, tagged) and cannot produce attach errors
- tmux attach now works correctly both inside tmux (switch-client) and outside tmux (attach-session)
- Spawning a new agent auto-switches the user to the new pane

## Open Tail

*(none)*

## Evidence

- transcript lines 1954-1956
- transcript lines 1992-2094
- transcript lines 2096-2121
- transcript lines 2157-2160
- transcript lines 2162-2183

