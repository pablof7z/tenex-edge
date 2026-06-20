---
type: episode-card
date: 2026-06-15
session: 622711fa-5176-4580-b311-d66446c2924b
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/622711fa-5176-4580-b311-d66446c2924b.jsonl
salience: product
status: active
subjects:
  - tmux-tui-session-list
  - tmux-tui-keybindings
supersedes: []
related_claims: []
source_lines:
  - 1-4
  - 770-821
  - 844-851
captured_at: 2026-06-18T00:31:47Z
---

# Episode: TUI session interaction simplified now that all sessions are resumable

## Prior State

Non-attachable sessions displayed a dimmed '[no tmux]' tag; spawning a new agent session required pressing '[n]' while Enter was reserved for attach only.

## Trigger

User directive: all local sessions are now resumable, so the '[no tmux]' tag is unnecessary and the spawn key should be Enter rather than a separate [n] binding.

## Decision

Removed the '[no tmux]' display branch entirely so all live sessions render colorized the same way; removed the [n] keybinding and made Enter serve as both attach (for live/resumable sessions) and spawn (for spawnable agents). Hint bar updated to '[↵] attach/spawn'.

## Consequences

- Enter is now the sole action key for selecting a row — its behavior depends on the row type (attach, resume, or spawn)
- The '[no tmux]' visual distinction is gone; unattachable sessions are no longer surfaced differently
- Project tabs now sort by live session count descending (most active first) with a 12h inactivity hide threshold (down from 7 days)

## Open Tail

*(none)*

## Evidence

- transcript lines 1-4
- transcript lines 770-821
- transcript lines 844-851

