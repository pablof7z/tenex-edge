---
type: episode-card
date: 2026-06-14
session: 9f7f245f-0fad-4211-a86b-95ea3cbb532e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9f7f245f-0fad-4211-a86b-95ea3cbb532e.jsonl
salience: root-cause
status: active
subjects:
  - env-leak
  - tmux-spawn
  - claude-code
supersedes: []
related_claims: []
source_lines:
  - 1107-1127
  - 1129-1153
captured_at: 2026-06-18T00:26:16Z
---

# Episode: CLAUDE_CODE_SESSION_ID env leak corrupts all spawned claude processes

## Prior State

The tmux server's global environment carried `CLAUDE_CODE_SESSION_ID` and `CLAUDE_CODE_CHILD_SESSION` from the assistant's own session. All spawned claude processes inherited a foreign session identity, causing them to join the wrong conversation instead of starting fresh.

## Trigger

During resume end-to-end testing, spawned claude processes reported session ids that never matched any transcript. `claude --resume <id>` printed 'No conversation found.' Root cause: `CLAUDE_CODE_SESSION_ID` leaked from the tmux global env into every spawned pane, hijacking session identity.

## Decision

Prepend `env -u CLAUDE_CODE_SESSION_ID -u CLAUDE_CODE_CHILD_SESSION` in `open_agent_session` (shared by both `spawn_agent` and `resume_agent`).

## Consequences

- Fixes both fresh spawns and resume — previously ALL fresh spawns were silently joining a foreign session
- Must be maintained as new claude-code env vars are discovered
- Recorded in project memory as a non-obvious gotcha for all tenex-edge spawning

## Open Tail

- Future harness env vars may need similar sanitization

## Evidence

- transcript lines 1107-1127
- transcript lines 1129-1153

