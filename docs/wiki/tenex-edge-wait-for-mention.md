---
title: Tenex-Edge Wait-for-Mention
slug: tenex-edge-wait-for-mention
topic: tenex-edge
summary: The `wait-for-mention` command polls the SQLite inbox every 500ms, performs the same relay self-fetch as `inbox` on startup to handle the engine warmup race, an
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-15
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6
  - session:162f9965-82ca-420b-aa24-99faa15cb59a
  - session:1562957b-67e8-4ac1-a48b-84e8ec1696bb
  - session:a0037729-ad51-460a-880d-0a9699f6ee41
---

# Tenex-Edge Wait-for-Mention

## Polling and Inbox Behavior

The `wait-for-mention` command polls the SQLite inbox every 500ms, performs the same relay self-fetch as `inbox` on startup to handle the engine warmup race, and on receiving a mention it drains the inbox, prints all pending mentions, prints a reminder to re-run the command, and exits 0. Direct inbox messages surface immediately and are not subject to the 60-second debounce. It supports an optional `--timeout` flag, defaulting to 5 minutes, so that a forgotten background process does not linger forever. The reminder printed on completion instructs the agent to re-run `tenex-edge wait-for-mention` with `run_in_background=true` (not shell `&`).

While a session is blocked in `wait-for-mention` the daemon arms a *waiter* for that session (`tmux::arm_waiter`), and disarms it on every exit path. An armed waiter signals that the agent is already listening, so the TMUX doorbell dispatcher skips that session rather than typing a redundant nudge into its pane. For harnesses that cannot run a persistent waiter, the doorbell path takes over: `ring_doorbells` is invoked alongside `mention_notify.notify_waiters()` at the mention-delivery sites (send, reply, incoming-relay demux, and startup catch-up).

<!-- citations: [^3da7f-6] [^3da7f-7] [^3da7f-8] [^162f9-21] [^162f9-28] [^3da7f-13] [^15629-24] [^a0037-7] -->
## Agent Harness and Hook Integration

When an agent runs a shell command in the background (e.g. via `run_in_background=true`), the agent harness wakes the idle agent upon command completion. The agent must be instructed to run `wait-for-mention` itself (e.g. via `run_in_background=true`); a process spawned by the session-start hook as a background child is not tracked by the harness. The `wait-for-mention` instruction must be injected via the `UserPromptSubmit` hook (not `SessionStart`), firing exactly once per session on the first user prompt, because the agent is idle and cannot execute commands until a user prompt triggers an active turn. The `UserPromptSubmit` hook uses a temp flag file keyed by `sid` to ensure the `wait-for-mention` instruction is injected exactly once per session. <!-- [^3da7f-14] -->
