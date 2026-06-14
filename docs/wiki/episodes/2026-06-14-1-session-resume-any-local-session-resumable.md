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
  - resume-id
supersedes:
  - 2026-06-14-1-resume-spawn-path-for-dead-agent
  - 2026-06-14-1-session-resume-per-harness-resume-commands
related_claims: []
source_lines:
  - 1-72
  - 74-77
  - 492-532
  - 844-857
  - 1298-1327
  - 1348-1397
captured_at: 2026-06-14T12:18:48Z
---

# Episode: Session resume: any local session resumable via harness-native token

## Prior State

No mechanism to reconstitute a conversation whose pane/process had died. Only `tmux attach` to a still-living pane, or doorbell injection into an idle-but-alive agent. SpawnDef only knew cold-start commands per harness. No per-harness resume command construction existed.

## Trigger

User asked whether tenex-edge could resume sessions (including ones not originally spawned via tenex-edge tmux), observing that most harnesses support resume commands. Later, user corrected that ALL local sessions should be resumable — not just dead ones — excoriating the 'skip alive sessions' gate.

## Decision

Built `tenex-edge tmux resume`: stores `resume_id` column in sessions table; per-harness resume command shapes keyed by the command binary (not the agent slug); `resume_agent` shares `open_agent_window` with `spawn_agent`; TUI shows 'Resumable' section and `[r]` key works on any row (Live `[no tmux]` or dead); CLI `tmux resume --session <id>`; RPCs `tmux_resume` and `tmux_resumable`. For claude/codex, `resume_id` falls back to `session_id` itself (native id = resume token). Local-only (host check). No first-prompt injection on resume.

## Consequences

- Resume shape keyed by binary not slug — custom agents like 'developer' (whose binary is claude) work automatically
- opencode plugin now forwards its native `ses_*` id as `resume_id`, but its sessions resume under a new `te-*` fabric identity (known limitation)
- The `resume_id` column was added via schema migration; existing sessions have empty resume_id and fall back to session_id for claude/codex
- Live sessions without a tmux pane (the '[no tmux]' rows) are now resumable via the same path, not just attachable
- Headless resume (no client attached) causes claude to exit after rendering — only affects automated testing, not real use

## Open Tail

- opencode id round-trip: the generated `te-*` id may not be accepted by opencode's own `--session` flag — needs empirical verification
- Cross-machine resume (ssh into remote) explicitly scoped out for now

## Evidence

- transcript lines 1-72
- transcript lines 74-77
- transcript lines 492-532
- transcript lines 844-857
- transcript lines 1298-1327
- transcript lines 1348-1397

