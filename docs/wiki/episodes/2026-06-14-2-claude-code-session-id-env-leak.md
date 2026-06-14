---
type: episode-card
date: 2026-06-14
session: 9f7f245f-0fad-4211-a86b-95ea3cbb532e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9f7f245f-0fad-4211-a86b-95ea3cbb532e.jsonl
salience: root-cause
status: superseded
subjects:
  - tmux-spawn
  - env-sanitization
  - claude-code-integration
supersedes: []
related_claims: []
source_lines:
  - 989-998
  - 1007-1131
  - 1133-1153
captured_at: 2026-06-14T12:18:48Z
---

# Episode: CLAUDE_CODE_SESSION_ID env leak corrupts spawned session identity

## Prior State

tenex-edge's `open_agent_window` spawned agent processes inheriting the tmux server's global environment unchanged. This went unnoticed because cold-start sessions would eventually get their own identity — but the hook-reported session_id never matched the actual transcript file.

## Trigger

During end-to-end resume testing, `claude --resume <id>` printed 'No conversation found' and exited. Investigation revealed spawned claude instances were joining the parent's session (9f7f245f) instead of starting fresh, because `CLAUDE_CODE_SESSION_ID` and `CLAUDE_CODE_CHILD_SESSION` leaked through tmux's global env. The hook-reported id (e.g. 70cda76e) never got a transcript — everything wrote into 9f7f245f.jsonl.

## Decision

`open_agent_window` now prepends `env -u CLAUDE_CODE_SESSION_ID -u CLAUDE_CODE_CHILD_SESSION` to all spawn and resume commands, stripping those env vars before launching the harness binary.

## Consequences

- Fresh spawns now correctly create their own session transcripts instead of hijacking the parent's
- Resume tokens now resolve to real conversation files instead of hitting 'No conversation found'
- All spawns (not just resume) were silently corrupted before this fix — the feature testing exposed a pre-existing bug

## Open Tail

*(none)*

## Evidence

- transcript lines 989-998
- transcript lines 1007-1131
- transcript lines 1133-1153

