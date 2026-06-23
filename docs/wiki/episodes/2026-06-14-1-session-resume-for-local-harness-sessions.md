---
type: episode-card
date: 2026-06-14
session: 9f7f245f-0fad-4211-a86b-95ea3cbb532e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9f7f245f-0fad-4211-a86b-95ea3cbb532e.jsonl
salience: product
status: superseded
subjects:
  - session-resume
  - spawn-def
  - tmux-harness
  - resume-id
supersedes: []
related_claims: []
source_lines:
  - 1-72
  - 852-853
  - 1322-1327
  - 1348-1365
captured_at: 2026-06-14T15:19:34Z
---

# Episode: Session Resume for Local Harness Sessions

## Prior State

Only living tmux panes could be reconnected to (via tmux attach). Dead sessions were gone forever. Sessions not originally spawned by tenex-edge tmux had no way to be pulled in. SpawnDef only knew cold-start commands per harness.

## Trigger

User asked whether tenex-edge could resume sessions, including ones not initiated via tenex-edge tmux, since most agent harnesses support resume commands.

## Decision

Built a full resume path: per-harness resume command construction (claude: --resume flag, codex: resume subcommand, opencode: --session flag), keyed by the command binary rather than the agent slug, local-only (host check), and working for ANY local session with a known id — not just dead ones. Added sessions.resume_id column; opencode plugin now forwards its native ses_* id; resume_token_for falls back to session_id itself for claude/codex (their adopted id IS the resume token).

## Consequences

- Resume scope broadened from dead-only to all-local after user rejected the dead-only gate — live [no tmux] sessions can now be pulled into tenex-edge tmux via resume
- Resume shape keyed by command binary (not agent slug) so custom agents like 'developer' automatically get the right resume syntax for their underlying harness
- Resumed sessions re-register on the fabric via SessionStart hook, re-binding their tmux endpoint to the new pane
- opencode sessions with synthetic te-* ids (where native ses_* was never captured) remain non-resumable — known limitation
- No first-prompt injection on resume; the harness restores its own context

## Open Tail

- opencode id round-trip: plugin forwards ses_* now but resumed opencode sessions get a new te-* fabric identity
- Resuming a session genuinely still alive elsewhere spawns a second instance on the same conversation — acceptable for stale terminals but worth noting

## Evidence

- transcript lines 1-72
- transcript lines 852-853
- transcript lines 1322-1327
- transcript lines 1348-1365

