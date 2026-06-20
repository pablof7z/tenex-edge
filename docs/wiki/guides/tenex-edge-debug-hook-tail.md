---
title: tenex-edge Debug Hook-Tail Command
slug: tenex-edge-debug-hook-tail
topic: tenex-edge
summary: The `tenex-edge debug hook-tail` command shows what was or will be injected in a session and by which hook, as well as what tenex-edge commands each session is
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-17
updated: 2026-06-17
verified: 2026-06-17
compiled-from: conversation
sources:
  - session:rollout-2026-06-17T10-44-56-019ed48a-54f6-7c41-a23b-dfde9dc65c2f
  - session:rollout-2026-06-17T10-51-45-019ed490-9414-75c3-ab93-66265458c6e9
---

# tenex-edge Debug Hook-Tail Command

## Overview

The `tenex-edge debug hook-tail` command shows what was or will be injected in a session and by which hook, as well as what tenex-edge commands each session is running or has run. It is a separate subcommand rather than overloading `tail --live`, keeping the existing fabric tail stream intact. The command must compile and route through `DebugAction::HookTail` and `cli::run`, exposing missing or invalid modules as build blockers rather than runtime mysteries. The implementation must include `src/cli/debug.rs` and `src/cli/command_forensics.rs` module files in the source tree before the build can succeed.

The command accepts `--project`, `--panes`, and `--session` (session ID or codename, e.g. `bravo4217`) flags. When an agent command does not pass `--session`, the debug view infers the pane from `TENEX_EDGE_AGENT` plus the current project when there is exactly one matching live session.

Hook context injections are logged as `context-injection` notes, and hook functions (`turn_start`/`turn_check`) return the emitted context text so the TUI can show which hook injected which text. The debug UI must distinguish between a CLI invocation that never parsed (e.g., old/hallucinated commands, bad flags, wrong subcommands) and a hook that parsed but failed open.

<!-- citations: [^rollo-98] [^rollo-99] [^rollo-100] [^rollo-106] -->
## TUI Interface

The debug hook-tail interface is a TUI built with ratatui that uses colors for its display. It shows panes of live sessions as a grid and allows the user to choose any pane to focus on it. Users can filter which projects and which sessions are shown, and can add or remove panes that show up in a window-tile-manager style grid. <!-- [^rollo-101] -->

The TUI supports the following controls: `Tab`/`Right` focus next pane, `Left` focus previous pane, `Enter`/`f` zoom, `+`/`-` change pane count, `p` cycle project filter, `s` cycle session filter, `a` clear filters, `q` quit. <!-- [^rollo-102] -->


The `debug hook-tail` feature must expose TUI pane issues—max pane count, refresh rate, text overflow, empty panes, stale tmux endpoints, missing tmux, and attach/resume failures—without breaking the TUI. Negative and live-state failure cases (no daemon, protocol skew, stale installed daemon, empty hook logs, unread inbox with no session, session in another project, remote session not resumable, command logging disabled, or unwritable path) must degrade with an explicit pane or status row. <!-- [^rollo-110] -->
## Data Sources

The TUI reader queries the daemon for current `who` rows when available so that quiet but live sessions still appear as panes, and falls back to showing forensic logs from disk if the daemon is down. The current v1 implementation polls local JSONL forensic files plus live `who` rows; the durable v2 architecture is daemon-ingested SQLite with a `debug_tail` streaming RPC. <!-- [^rollo-103] -->

The `debug hook-tail` feature must expose the existing hook-call forensic stream (`hook-calls.jsonl`) rather than inferring behavior from tail's fabric events alone. It must label hook-call telemetry and fabric `tail` events separately to avoid confusing the two streams. <!-- [^rollo-107] -->

The feature must log malformed hook stdin cases including invalid JSON, empty stdin, unknown host, unknown hook type, and missing session IDs for Claude/Codex. It must show both that a hook was called and whether it produced a visible context injection, distinguishing plain text, Codex `systemMessage`, or Claude `hookSpecificOutput` output formats. <!-- [^rollo-108] -->

The feature must cover the full inbox send/drain lifecycle including `inbox send`, local delivery, tail `Msg`/`Sync`, `inbox` drain, `wait-for-mention`, and reply-by-ID. The implementation must correctly distinguish between draining (for `user-prompt-submit`) and peeking (for `post-tool-use`), with tmux injection marking the exact rows delivered. <!-- [^rollo-109] -->
