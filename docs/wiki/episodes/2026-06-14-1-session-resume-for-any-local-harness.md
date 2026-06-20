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
  - resume-token
supersedes: []
related_claims: []
source_lines:
  - 1-77
  - 74-77
  - 1298-1327
  - 1348-1447
captured_at: 2026-06-18T00:26:16Z
---

# Episode: Session resume for any local harness session

## Prior State

SpawnDef only knew cold-start commands per harness (e.g. `claude`, `codex`). The only reconnection actions were tmux attach to a living pane or doorbell injection into an idle agent. Dead sessions were permanently lost. Resume was scoped to dead sessions only, and resume required a separately-stored `resume_id` column.

## Trigger

User asked whether sessions could be resumed from tenex-edge tmux. Then user corrected the initial dead-only scope: 'if we have the session id and its in the same computer it should be resumable' — any local session, including live `[no tmux]` rows.

## Decision

Added a resume spawn path with per-harness resume command templates keyed by the harness *binary* (not agent slug): `claude`→`--resume <id>`, `opencode`→`--session <id>`, `codex`→`resume <id>` (subcommand). Resume token falls back to `session_id` itself for claude/codex (their adopted native id IS the resume token). Local-only (host check). No first-prompt injection on resume. Exposed via CLI (`tenex-edge tmux resume`), TUI (`[r]` on any local row), and RPCs (`tmux_resume`, `tmux_resumable`). opencode's plugin now forwards its native `ses_*` id.

## Consequences

- Resume works for both dead AND live local sessions (live ones spawn a second instance on the same conversation — known edge case)
- opencode resumes under a new `te-*` fabric identity because the plugin can't pre-seed its id — noted limitation
- Resume deliberately local-only; no remote machine SSH support
- Slug≠binary bug surfaced and fixed: agent slug 'developer' maps to binary 'claude', resume shapes must key on binary
- Pre-existing sessions without a stored `resume_id` now qualify because the token falls back to `session_id`

## Open Tail

- opencode id round-trip: the plugin forwards `ses_*` but can't pre-seed it, so resumed opencode sessions get a new fabric identity

## Evidence

- transcript lines 1-77
- transcript lines 74-77
- transcript lines 1298-1327
- transcript lines 1348-1447

