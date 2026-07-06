---
title: Tenex-Edge Launch
slug: tenex-edge-launch
topic: tenex-edge
summary: When `tenex-edge launch` spawns an agent harness in tmux, `default-terminal` and `terminal-overrides` are set globally (`-g`) before `new-session` forks the chi
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-03
updated: 2026-07-06
verified: 2026-07-06
compiled-from: conversation
sources:
  - session:abce9e9f-8f3e-4561-9dd3-684afd59be80
  - session:fea5307b-d9a0-46fe-977c-408e5e0e0ff4
---

# Tenex-Edge Launch

## Named Commands

Agent launch commands are stored as named entries in `~/.tenex-edge/agents/<slug>.json` under `commands`, for example `{"commands":[{"name":"full","argv":["claude","--dangerously-skip-permissions"]}]}`. The removed singular `command` field is ignored by launch resolution; it is not read as a legacy fallback.

`tenex-edge launch <agent>` chooses the command before calling the daemon. A single configured command launches directly. Multiple configured commands open a TTY picker unless `--command-name <name>` selects one explicitly. `-c/--command <command>` remains a one-shot full argv override. If no commands exist, interactive launch suggests commands from other agents' `commands` entries with conservative slug/path adaptation; if no local suggestions exist, it suggests built-in harness commands.

## tmux Environment Inheritance

When `tenex-edge launch` spawns an agent harness in tmux, `default-terminal` and `terminal-overrides` are set globally (`-g`) before `new-session` forks the child process so the child inherits `tmux-256color` and color support from the first frame. The `terminal-overrides` value uses the format `*:Tc:RGB:extkeys` (term-pattern-prefixed capability tokens), not the previous orphaned format `,*:Tc,RGB,extkeys`.

With these settings in place, the fixed `tenex-edge launch` window renders agent harness colors identically to a direct launch — including the colored robot logo, orange banner, and yellow warning.

`@te_session` is a tmux session option stamped by the daemon that the agent harness expects to be set. When correctly configured, the status line reads `claude@isolated-test-mac project project [idle]` instead of showing a red `@te_session not set` error.

The `tenex-edge launch` status line displays the agent identifier and session state in the format `<agent>@<host> <project> <project> [idle]` (e.g. `claude@isolated-test-mac project project [idle]`).

<!-- citations: [^abce9-cad96] [^abce9-0fd96] [^abce9-2b259] [^abce9-b1683] -->

## Mention Injection

Mention injection into a `tenex-edge launch` agent must occur in the `userPrompt` hook. <!-- [^fea53-66223] -->

## Agent Roles

Agent1 is a raw `claude` session (direct mode, host-tmux-observed but not daemon-anchored) that the user types the initial instruction into. Agent2 is a `claude` session launched via `tenex-edge launch` that is expected to receive live daemon-pushed, attributed mentions. <!-- [^fea53-1e000] -->
