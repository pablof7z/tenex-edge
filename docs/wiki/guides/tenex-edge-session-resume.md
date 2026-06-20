---
title: tenex-edge Session Resume
slug: tenex-edge-session-resume
topic: tenex-edge
summary: "Session resume is local-only: resuming a remote machine's session requires SSH and is out of scope"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-14
updated: 2026-06-15
verified: 2026-06-14
compiled-from: conversation
sources:
  - session:9f7f245f-0fad-4211-a86b-95ea3cbb532e
  - session:0afc3cf4-3465-4b37-a7ec-63b798d78621
  - session:622711fa-5176-4580-b311-d66446c2924b
---

# tenex-edge Session Resume

## Scope & Constraints

Session resume is local-only: resuming a remote machine's session requires SSH and is out of scope. The only sessions that remain non-resumable are opencode sessions whose `ses_*` id was never captured (synthetic `te-` ids) and remote sessions (different machine). <!-- [^9f7f2-1] -->

## Resume Command Construction

The resume command runs in a new tmux pane/window, composing the agent's own launch config (e.g., `claude --dangerously-skip-permissions`) with the harness-specific resume token. The command shape is keyed by the command binary (e.g., `claude`), not the agent slug (e.g., `developer`), so custom agents inherit the correct harness resume syntax automatically. Per-harness resume command construction uses `append-flag` shape for claude (`--resume <id>`) and opencode (`--session <id>`), and `subcommand` shape for codex (`resume <id>` with flags on the subcommand). Resume does not inject a first prompt; the harness restores its own context from the resume token. For newly spawned sessions, the first prompt depends on the trigger: manual spawns from the TUI inject nothing (they start clean), while mention-triggered spawns (p-tagged in a message) inject the actual received message instead of a hardcoded trigger string.

<!-- citations: [^9f7f2-2] [^0afc3-1] -->
## Resume Token Storage & Fallback

The `sessions` table stores a `resume_id` column: for claude/codex it equals the adopted native session id, and for opencode it holds the forwarded `ses_*` id. The opencode plugin forwards its native `ses_*` session id (read from `lastUser.info.sessionID`) in the hook payload so it can be stored as the resume token. The resume token falls back to the `session_id` itself for harness-native ids (claude/codex), so sessions without a separately-stored `resume_id` still qualify as resumable. <!-- [^9f7f2-3] -->

## TUI Resumability

Any local session with a known id is resumable via the TUI `[r]` key or Enter key, including Live `[no tmux]` sessions, not just fully-dead sessions. Resuming a session that is genuinely still running elsewhere spawns a second instance on the same conversation, which is acceptable for stale `[no tmux]` rows (lost terminals) but worth noting. <!-- [^9f7f2-4] -->

## Tmux Isolation

Each spawned/resumed agent runs in its own independent tmux session (`te-<agent>`) rather than a window in a shared `tenex` session, so attaching to one agent does not pull in others. <!-- [^9f7f2-5] -->

## Environment & Identity

Spawned agent windows strip `CLAUDE_CODE_SESSION_ID` and `CLAUDE_CODE_CHILD_SESSION` from the environment to prevent the agent from hijacking a foreign session. OpenCode sessions resumed under the feature use a new `te-*` fabric identity because the plugin cannot pre-seed its id, which is a known limitation. <!-- [^9f7f2-6] -->

## First Prompt Injection

Creating a new session with tenex-edge tmux sends no automatic user message (including no automatic 'tenex-edge inbox' message). When a session is spawned because an agent was p-tagged in a message, the daemon types the actual received message as the first prompt. The injected message is rendered through the same envelope formatter the inbox uses (sender, reply ID, body). The mention's inbox row is persisted as already-delivered so that the turn-start inbox drain does not re-inject it as duplicate context; however, the already-delivered row remains resolvable by `inbox reply --id` to preserve reply threading. Multi-line messages are injected via a tmux paste buffer with bracketed-paste mode (`paste-buffer -p`) followed by a single Enter to submit, preventing premature submission after the first line.

<!-- citations: [^0afc3-2] [^62271-5] -->
