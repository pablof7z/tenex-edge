---
type: episode-card
date: 2026-06-14
session: 9f7f245f-0fad-4211-a86b-95ea3cbb532e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9f7f245f-0fad-4211-a86b-95ea3cbb532e.jsonl
salience: architecture
status: active
subjects:
  - tmux-session-model
  - agent-spawn
supersedes: []
related_claims: []
source_lines:
  - 1912-1968
  - 2163-2197
captured_at: 2026-06-18T00:26:16Z
---

# Episode: Per-agent independent tmux sessions replace shared session

## Prior State

All spawned/resumed agents were windows in one shared `tenex` tmux session. Attaching to one agent's pane showed all agents' windows. The `ensure_view_session` machinery created grouped per-client views to avoid mirroring, but had a reaping race (`destroy-unattached on` destroyed the view before attach could complete, causing 'can't find session' errors).

## Trigger

User: 'all the attaching of tmux sessions need to happen in independent panes, right now all attached sessions are in the same session; ideally it would attach in a completely independent session'

## Decision

Each spawned/resumed agent gets its own independent tmux session (`te-<slug>`, `te-<slug>-<N>`, etc.). Deleted the entire grouped-view machinery (`ensure_view_session`, `destroy-unattached`, view session creation/management). `open_agent_window` renamed to `open_agent_session` and creates a new detached session per agent. Attach is now a simple `tmux attach-session -t te-<agent>`.

## Consequences

- No more mirroring between clients viewing the same session
- Simpler attach code — no view session lifecycle, no reaping race
- Old `tenex` shared session left as-is for pre-change agents still running
- Each agent session has exactly one window; window names carry agent identity

## Open Tail

*(none)*

## Evidence

- transcript lines 1912-1968
- transcript lines 2163-2197

