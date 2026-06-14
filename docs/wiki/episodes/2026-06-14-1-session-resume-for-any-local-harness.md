---
type: episode-card
date: 2026-06-14
session: 9f7f245f-0fad-4211-a86b-95ea3cbb532e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9f7f245f-0fad-4211-a86b-95ea3cbb532e.jsonl
salience: product
status: active
subjects:
  - session-resume
  - spawn-def
  - tmux-harness
  - resume-shape
supersedes:
  - 2026-06-14-1-session-resume-for-local-harness-sessions
  - 2026-06-14-1-session-resume-any-local-session-resumable
related_claims: []
source_lines:
  - 1-72
  - 74-76
  - 80-145
  - 840-870
  - 1298-1327
  - 1348-1450
captured_at: 2026-06-14T15:25:14Z
---

# Episode: Session resume for any local harness session

## Prior State

SpawnDef only knew cold-start commands per harness (e.g. `claude`, `codex`). The only reconnection action was `tmux attach` to a still-living pane, or injecting into an idle-but-alive agent via doorbell. No mechanism existed to reconstitute a conversation whose pane/process had died. The initial implementation also gated resume to dead sessions only, showing 'cannot attach' for live sessions with `[no tmux]`.

## Trigger

User asked whether tenex-edge could resume sessions — including ones not originally spawned by it. Then strongly corrected that ANY local session with a known ID should be resumable, not just dead ones.

## Decision

Added `tenex-edge tmux resume` (CLI, TUI, RPC) that reconstitutes a session by replaying the harness with its native resume token in a new tmux window. Resume shape is keyed by the command binary (`claude`→`--resume <id>` flag, `opencode`→`--session <id>` flag, `codex`→`resume <id>` subcommand), not the agent slug, so custom agents work automatically. Resume is available for ALL local sessions (alive or dead), and Enter on a `[no tmux]` row resumes instead of dead-ending. Resume token falls back to `session_id` itself for claude/codex (where the adopted native id IS the resume token). Local-only (host check). No first-prompt injection on resume.

## Consequences

- Opencode's native `ses_*` id is now forwarded from the plugin (was read but never sent), enabling resume for opencode sessions that went through the hook
- Opencode sessions created before this change still carry synthetic `te-*` ids and remain non-resumable
- Resuming a session that is genuinely still running elsewhere will spawn a second instance on the same conversation
- The `sessions.resume_id` column stores the harness-native resume token separately from `session_id`
- Custom agents (like `developer` whose binary is `claude`) automatically get the correct resume shape

## Open Tail

- User's last request: attached tmux sessions should open in completely independent tmux sessions, not grouped views in the same session

## Evidence

- transcript lines 1-72
- transcript lines 74-76
- transcript lines 80-145
- transcript lines 840-870
- transcript lines 1298-1327
- transcript lines 1348-1450

