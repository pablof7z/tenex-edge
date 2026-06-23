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
supersedes: []
related_claims: []
source_lines:
  - 1-77
captured_at: 2026-06-14T11:15:26Z
---

# Episode: Resume-spawn path for dead agent sessions

## Prior State

SpawnDef only knew cold-start commands per harness (e.g. `claude`, `codex`). The only reconnection action was `tmux attach` to a still-living pane, or injecting into an idle-but-alive agent via doorbell. No mechanism existed to reconstitute a conversation whose pane/process had died.

## Trigger

User observed that most agent harnesses support resuming sessions and asked whether tenex-edge could resume sessions — including ones not originally spawned via tenex-edge tmux. Investigation revealed the harness-native session IDs are already stored because of the adopt-native-id design (claude/codex), making the resume token available without architectural change.

## Decision

Add a new 'resume' spawn path distinct from 'attach': when a session's pane is dead but its native ID exists in the sessions table, spawn a new tmux pane running the harness-specific resume command (e.g. `claude --dangerously-skip-permissions --resume <id>`, `codex --some-parameter resume <id>`). Scoped to local machine only — no remote-machine resume. Per-harness resume command templates must be added to SpawnDef, and a liveness check must determine attach-vs-resume.

## Consequences

- SpawnDef/HostDef must be extended with per-harness resume-command templates alongside existing cold-start commands
- A liveness signal (does the tmux pane still exist?) is needed to choose between attach and resume actions in the TUI
- For claude-code and codex, resume tokens are already stored (generates_sid: false); opencode's generated IDs must be empirically verified to round-trip through its own resume command
- Resumed sessions are brand-new processes — open question whether they reuse the original session row/shortcode or create a new one pointing back (affects inbox routing and fabric listing)
- Resume action design should not bake in local-only assumptions about pane access, since cross-machine resume has the same shape and may be wanted later

## Implementation (built + validated 2026-06-14)

Shipped and tested end-to-end with a real `developer` (claude) session: spawn → converse (codeword) → kill pane → `tenex-edge tmux resume` → conversation fully restored, session re-registered on the fabric.

- **Resume token storage**: `sessions.resume_id` column (migration). For claude/codex it equals the adopted `session_id`; for opencode it's the forwarded `ses_*` id. Populated in `rpc_session_start` (harness-supplied id ⇒ resume_id) and via a new `resume_id` field on the session-start/user-prompt-submit hook payload. The opencode plugin now forwards `ocSessionID` (it already had it; it just wasn't sent).
- **Resume-command shape** keyed by the launch command's *binary*, NOT the agent slug — `resume_shape_for_bin()`: `claude`→`--resume <id>` (append flag), `opencode`→`--session <id>` (append flag), `codex`→`resume <id>` (subcommand after argv0, flags ride after). `build_resume_command()` is pure + unit-tested. Keying by binary is essential: custom agents like `developer` (binary `claude`) would otherwise miss.
- **Spawn path**: `resume_agent()` shares `open_agent_window()` with `spawn_agent()`; resume injects NO first prompt (harness restores its own context). RPCs `tmux_resume` + `tmux_resumable`; CLI `tenex-edge tmux resume --session <id>`; TUI gains a "Resumable (no live pane)" section with `[r]`/Enter. Local-machine only (host check).
- **Critical fix found by actually running it**: the tmux server's global env leaks `CLAUDE_CODE_SESSION_ID` / `CLAUDE_CODE_CHILD_SESSION` into spawned panes, so a fresh `claude` hijacks a foreign session (its hook-reported id never gets a transcript) and `--resume` collides. `open_agent_window` now prepends `env -u CLAUDE_CODE_SESSION_ID -u CLAUDE_CODE_CHILD_SESSION` to every spawned/resumed command. This made both fresh-spawn identity AND resume correct.
- **opencode round-trip**: resolved by capture, not generation — the `ses_*` id is opencode's documented `--session` token; we capture it rather than relying on our synthetic `te-*`.
- **Session identity on resume**: claude/codex report the SAME id on `--resume`, so the original session row + shortcode are reused (endpoint re-binds to the new pane automatically). opencode mints a new `te-*` on resume (plugin can't pre-seed its id), so an opencode resume continues the conversation but under a new fabric identity — known limitation.

## Evidence

- transcript lines 1-77 (design); implementation in src/tmux.rs, src/state.rs, src/daemon/server.rs, src/daemon/server/tmux_rpc.rs, src/cli/hooks.rs, src/cli/tmux_cli.rs, integrations/opencode/tenex-edge.ts

