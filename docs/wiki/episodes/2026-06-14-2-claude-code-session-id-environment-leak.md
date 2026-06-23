---
type: episode-card
date: 2026-06-14
session: 9f7f245f-0fad-4211-a86b-95ea3cbb532e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9f7f245f-0fad-4211-a86b-95ea3cbb532e.jsonl
salience: root-cause
status: active
subjects:
  - env-leak
  - spawn-path
  - claude-code
supersedes:
  - 2026-06-14-2-claude-code-session-id-env-leak
related_claims: []
source_lines:
  - 1107-1127
captured_at: 2026-06-14T15:19:34Z
---

# Episode: CLAUDE_CODE_SESSION_ID Environment Leak Corrupting All Spawns

## Prior State

Assumed that spawning `claude` in a new tmux pane would start a fresh session. The hook-reported session id was trusted as the real conversation id.

## Trigger

Testing resume failed with 'No conversation found' for the captured id. Investigation revealed that ALL spawned claude sessions were inheriting CLAUDE_CODE_SESSION_ID and CLAUDE_CODE_CHILD_SESSION from the tmux server's global environment, causing them to join an existing foreign conversation instead of starting fresh. The hook-reported id never got its own transcript — the real transcript was under a different id.

## Decision

Prepend `env -u CLAUDE_CODE_SESSION_ID -u CLAUDE_CODE_CHILD_SESSION` in `open_agent_window` so spawned processes start with a clean session identity.

## Consequences

- This bug was silently corrupting ALL fresh spawns — every claude session launched by tenex-edge was hijacking an existing conversation rather than starting its own
- Resume now works correctly because the spawned `claude --resume` actually creates/reads the transcript matching the captured id
- The captured session id now matches the real transcript id (verified: codeword EMERALD-RIVER-31 appeared in the transcript file named after the captured id)

## Open Tail

*(none)*

## Evidence

- transcript lines 1107-1127

