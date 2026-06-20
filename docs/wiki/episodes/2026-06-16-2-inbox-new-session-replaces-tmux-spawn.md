---
type: episode-card
date: 2026-06-16
session: a88513d3-754f-4369-b440-72c8d29331e2
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a88513d3-754f-4369-b440-72c8d29331e2.jsonl
salience: product
status: active
subjects:
  - cli-inbox-new-session
  - cli-tmux-spawn
supersedes: []
related_claims: []
source_lines:
  - 409-413
  - 667-709
captured_at: 2026-06-18T00:40:35Z
---

# Episode: inbox new-session replaces tmux spawn as CLI surface

## Prior State

Spawning a new agent session required `tenex-edge tmux spawn --agent <slug>`, a subcommand under the `tmux` group (which is an internal control-plane namespace).

## Trigger

User's forceful correction when the assistant assumed `tenex-edge inbox new-session --agent` didn't exist: 'for fuck's sake! tenex-edge inbox new-session --agent is a new fucking command' — meaning it should be created as the user-facing surface, not treated as a mistake.

## Decision

Created `tenex-edge inbox new-session --agent <slug> [--project <slug>]` as the primary CLI command for starting new sessions, delegating to the existing `tmux_spawn` daemon RPC. Removed `tenex-edge tmux spawn` CLI subcommand entirely (`TmuxAction::Spawn` variant and `tmux_spawn` fn in `tmux_cli.rs` deleted). Daemon RPC and TUI spawn path remain untouched.

## Consequences

- Agent-facing `who` output and help text reference `inbox new-session` instead of `tmux spawn`
- The `tmux` CLI group no longer exposes `spawn`; `tmux spawn` returns 'unrecognized subcommand'
- New `InboxAction::NewSession` variant wired in `cli.rs` dispatch, calling `messaging::new_session()`
- Daemon-side `rpc_tmux_spawn` and TUI spawn path are unchanged — only the CLI surface moved

## Open Tail

*(none)*

## Evidence

- transcript lines 409-413
- transcript lines 667-709

