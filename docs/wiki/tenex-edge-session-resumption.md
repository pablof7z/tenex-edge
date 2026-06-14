---
title: Tenex-Edge Session Resumption
slug: tenex-edge-session-resumption
topic: session-resumption
summary: Session resumption is limited to sessions on the current machine; resuming a remote machine's session requires SSH and is out of scope
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

Session resumption is limited to sessions on the current machine; resuming a remote machine's session requires SSH and is out of scope. Any local session with a known resume token is resumable, regardless of liveness, including live sessions currently running outside of tenex-edge tmux (tagged `[no tmux]`). Resuming a session runs the harness's resume command in a new tmux pane/window, including whatever config flags the local developer uses (e.g. `claude --dangerously-skip-permissions --resume <session-id>`). When spawning or resuming an agent pane, the environment must strip `CLAUDE_CODE_SESSION_ID` and `CLAUDE_CODE_CHILD_SESSION` (via `env -u`) to prevent the spawned process from hijacking a foreign session. Resuming a session that is genuinely still running elsewhere spawns a second instance on the same conversation. The CLI surface for resume is `tenex-edge tmux resume --session <id-or-prefix>`.

<!-- citations: [^9f7f2-6] [^9f7f2-8] [^9f7f2-12] -->
## Provider-Specific Resume Syntax

Resume command construction is keyed by the harness binary (e.g., `claude`, `codex`), not the agent slug (e.g., `developer`), so custom agents inherit the correct resume behavior automatically. The resume command shapes are: `claude` appends `--resume <id>` as a flag, `opencode` appends `--session <id>` as a flag, and `codex` uses `resume <id>` as a subcommand with flags after it. The `sessions` table stores a `resume_id` column distinct from the identity `session_id`: for claude/codex `resume_id == session_id`, and for opencode `resume_id` is the forwarded `ses_*` token. The resume token falls back to the `session_id` for claude/codex when a separately-stored `resume_id` is absent, so sessions lacking a stored `resume_id` still qualify for resumption. Opencode sessions whose `ses_*` token was never captured remain non-resumable, since their synthetic `te-*` ids are not valid resume tokens. The opencode plugin forwards its native `ses_*` session id (read from `lastUser.info.sessionID`) as the `resume_id` in the hook payload, enabling opencode session resumption. Resuming an opencode session restores the conversation but creates a new `te-*` fabric identity (the plugin cannot pre-seed its id).

<!-- citations: [^9f7f2-10] [^9f7f2-7] [^9f7f2-9] [^9f7f2-13] -->
## TUI and RPC Interface

The TUI displays a 'Resumable (no live pane)' section with `[r]`/Enter affordance for resuming sessions. The `r` key and Enter in the TUI resume the selected row uniformly, covering both live `[no tmux]` sessions and dead resumable sessions. The RPCs `tmux_resume` and `tmux_resumable` are added for the resume feature. <!-- [^9f7f2-11] -->
