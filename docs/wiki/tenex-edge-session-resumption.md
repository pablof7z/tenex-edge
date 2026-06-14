---
title: Tenex-Edge Session Resumption
slug: tenex-edge-session-resumption
topic: session-resumption
summary: Session resume is local-only â a session can only be resumed on the same machine where it ran, not on a remote machine
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-14
updated: 2026-06-14
verified: 2026-06-14
compiled-from: conversation
sources:
  - session:9f7f245f-0fad-4211-a86b-95ea3cbb532e
---

# Tenex-Edge Session Resumption

## Session Resumption

Session resume is local-only — a session can only be resumed on the same machine where it ran, not on a remote machine. The resume command reuses the agent's own launch config (from agent.json's command field) transformed per-harness, running in a new tmux pane/window. Any local session with a known resume token is resumable, regardless of liveness, including live sessions currently running outside of tenex-edge tmux (tagged `[no tmux]`). SpawnDef carries a per-harness resume spec (shape: append-flag vs subcommand, plus the token string) alongside the existing launch command. When spawning or resuming an agent pane, the environment must strip `CLAUDE_CODE_SESSION_ID` and `CLAUDE_CODE_CHILD_SESSION` (via `env -u`) to prevent the spawned process from hijacking a foreign session. Resuming a session that is genuinely still running elsewhere spawns a second instance on the same conversation. The CLI surface for resume is `tenex-edge tmux resume --session <id-or-prefix>`.

<!-- citations: [^9f7f2-16] [^9f7f2-6] [^9f7f2-8] [^9f7f2-12] [^9f7f2-22] -->
## Provider-Specific Resume Syntax

Resume command construction is keyed by the harness binary (e.g., `claude`, `codex`), not the agent slug (e.g., `developer`), so custom agents inherit the correct resume behavior automatically. The resume command shapes are: `claude` appends `--resume <id>` as a flag, `opencode` appends `--session <id>` as a flag, and `codex` uses `resume <id>` as a subcommand with flags after it. The `sessions` table stores a `resume_id` column distinct from the identity `session_id`: for claude/codex `resume_id == session_id`, and for opencode `resume_id` is the forwarded `ses_*` token. When `resume_id` is empty, the resume token falls back to the `session_id` itself (for harnesses like claude/codex where the native id is the resume token), so sessions lacking a stored `resume_id` still qualify for resumption. Opencode sessions whose `ses_*` token was never captured remain non-resumable, since their synthetic `te-*` ids are not valid resume tokens. The opencode plugin forwards its native `ses_*` session id (read from `lastUser.info.sessionID`) as the `resume_id` in the hook payload, enabling opencode session resumption. Resuming an opencode session restores the conversation but creates a new `te-*` fabric identity (the plugin cannot pre-seed its id).

The `open_agent_window` function sanitizes the tmux environment by prepending `env -u CLAUDE_CODE_SESSION_ID -u CLAUDE_CODE_CHILD_SESSION` to prevent spawned claude panes from hijacking a foreign session.

The per-client view session for attach uses a `client-detached` hook instead of `destroy-unattached on`, preventing tmux from reaping the session before the client can attach. The `has-session` existence check in `ensure_view_session` has its stderr silenced so it does not print `can't find session` noise to the terminal. An empty `TMUX` environment variable is treated as not-in-tmux.

<!-- citations: [^9f7f2-18] [^9f7f2-19] [^9f7f2-10] [^9f7f2-7] [^9f7f2-9] [^9f7f2-13] [^9f7f2-23] -->
## TUI and RPC Interface

The TUI displays a 'Resumable (no live pane)' section with `[r]`/Enter affordance for resuming sessions. The `r` key and Enter in the TUI resume the selected row uniformly, covering both live `[no tmux]` sessions and dead resumable sessions. The RPCs `tmux_resume` and `tmux_resumable` are added for the resume feature.

<!-- citations: [^9f7f2-11] [^9f7f2-17] -->

## Implementation History

The session-resume feature is committed on branch `session-resume` (commit `d08825a0`, 12 files, +778/−88) and pushed to `master`. <!-- [^9f7f2-20] -->

## Session Isolation

Each attached agent opens in its own independent tmux session rather than as a window in a shared tenex session. <!-- [^9f7f2-24] -->
