---
type: episode-card
date: 2026-06-14
session: 9f7f245f-0fad-4211-a86b-95ea3cbb532e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9f7f245f-0fad-4211-a86b-95ea3cbb532e.jsonl
salience: architecture
status: active
subjects:
  - tmux-session-layout
  - agent-isolation
supersedes: []
related_claims: []
source_lines:
  - 1912-1968
captured_at: 2026-06-14T15:30:57Z
---

# Episode: Per-agent independent tmux sessions replace shared window model

## Prior State

All spawned agents were windows inside a single shared `tenex` tmux session. Attaching to any agent dropped the user into that shared session, where all agents' windows were visible. The `open_agent_window` function ensured a single `tenex` session existed and created windows within it.

## Trigger

User directive: 'the attaching of tmux sessions need to happen in independent panes, right now all attached sessions are in the same session; ideally it would attach in a completely independent session'

## Decision

Replaced the shared-window model with per-agent independent tmux sessions. `open_agent_window` renamed to `open_agent_session`, now creates a dedicated detached tmux session per agent (named e.g. `developer·tenex-edge`). Attaching to an agent connects to its own session, isolating it from other agents.

## Consequences

- Each agent gets its own tmux session — no cross-visibility of other agents' windows when attached
- The `tenex` shared session is no longer created or referenced
- Attach no longer needs grouped view sessions for isolation since each agent is already in its own session

## Open Tail

- Implementation builds cleanly but end-to-end testing was not completed before transcript ended

## Evidence

- transcript lines 1912-1968

