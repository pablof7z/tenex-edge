---
title: Mosaico Launch
slug: mosaico-launch
topic: mosaico
summary: When `mosaico launch` spawns an agent harness in pty, `default-terminal` and `terminal-overrides` are set globally (`-g`) before `new-session` forks the chi
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-03
updated: 2026-07-15
verified: 2026-07-15
compiled-from: conversation
sources:
  - session:abce9e9f-8f3e-4561-9dd3-684afd59be80
  - session:fea5307b-d9a0-46fe-977c-408e5e0e0ff4
---

# Mosaico Launch

## Configured Harness

Mosaico monitors native agent definitions installed globally and in bound
workspaces: Codex `agents/*.toml`, Claude Code `agents/**/*.md`, and OpenCode
`agents/*.md`. Valid profiles join the backend's available-agent roster without
a duplicate Mosaico agent JSON. Workspace definitions override global definitions
for the same harness and slug and are advertised only to that workspace.

The corresponding `harnesses.json` bundle owns the underlying harness, transport,
and operational args. Mosaico selects the one compatible native bundle; ambiguous
bundles or a slug installed in multiple harnesses require an explicit agent JSON
binding. Claude and OpenCode PTY launches apply the discovered name with `--agent`.
Codex custom-agent TOML is resolved as a root-session configuration layer: app-server
and PTY launches stage its developer instructions and config into an isolated
`CODEX_HOME`.

An explicit agent file selects a required harness bundle and may select a separate
harness-specific config profile, for example
`{"slug":"reviewer","harness":"yolo-codex","profile":"deep"}`. Claude PTY/headless
applies such a profile with `--agent`; Codex PTY/headless applies it with `--profile`;
Codex app-server stages `$CODEX_HOME/<profile>.config.toml`. A Codex config profile is
not a Codex custom-agent TOML. Unsupported combinations fail loudly; an absent
profile activates a matching discovered native profile or uses the harness default.

A bare `mosaico launch <agent>` first checks whether `<agent>` is an existing public session handle. A live PTY is reattached; an exited session with a native resume token is resumed. Otherwise the daemon resolves either the explicit agent binding or the discovered native profile. The executable and profile mechanism are code-owned by the `(harness, transport)` driver; the bundle contributes operational args. There are no command pickers, launch-time command or bundle overrides, or built-in bundle fallbacks.

## Initial Prompt

`mosaico launch <agent> "prompt"` spawns the PTY session, waits for the harness input to become interactive, then injects and submits the prompt as the opening user turn before attaching the terminal. Omit the prompt for the normal clean interactive launch. Use `-` as the prompt value to read the opening prompt from stdin.

## pty Environment Inheritance

When `mosaico launch` spawns an agent harness in pty, `default-terminal` and `terminal-overrides` are set globally (`-g`) before `new-session` forks the child process so the child inherits `pty-256color` and color support from the first frame. The `terminal-overrides` value is `*:Tc:RGB:extkeys`.

With these settings in place, the fixed `mosaico launch` window renders agent harness colors identically to a direct launch — including the colored robot logo, orange banner, and yellow warning.

The daemon derives the session key before spawn and injects
`MOSAICO_PUBKEY` from process birth. PTY endpoint IDs, harness-native resume
tokens, sockets, and PIDs remain typed runtime locators owned by that pubkey;
none of them is a second session identity. Ordinary per-session agent configs do
not persist secret or public keys; only explicit `perSessionKey:false` agents keep
a durable keypair in their Mosaico agent JSON.

The `mosaico launch` status line displays the published agent name, work-root
channel, current channel, optional distilled title, and live state. For example:
`amber-claude mosaico support [Refactoring the inbox] [writing tests]`.

<!-- citations: [^abce9-cad96] [^abce9-0fd96] [^abce9-2b259] [^abce9-b1683] -->

## Mention Injection

Mention injection into a `mosaico launch` agent must occur in the `userPrompt` hook. <!-- [^fea53-66223] -->

## Agent Roles

Agent1 is a raw `claude` session (direct mode, host-pty-observed but not daemon-anchored) that the user types the initial instruction into. Agent2 is a `claude` session launched via `mosaico launch` that is expected to receive live daemon-pushed, attributed mentions. <!-- [^fea53-1e000] -->
