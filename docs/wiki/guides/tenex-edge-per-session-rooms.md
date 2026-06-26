---
title: Tenex-Edge Per-Session Rooms
slug: tenex-edge-per-session-rooms
topic: tenex-edge
summary: Per-session rooms can be disabled via a configuration flag.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-26
updated: 2026-06-26
verified: 2026-06-26
compiled-from: conversation
sources:
  - session:8510c3cc-9722-47a4-90ee-2f489646f5b8
---

# Tenex-Edge Per-Session Rooms

## Configuration

Per-session rooms can be disabled via a configuration flag. <!-- [^8510c-0226c] -->

## Behavior when disabled

When per-session rooms are disabled:

- Running `tenex-edge launch` without `--channel` in a TTY environment opens the interactive channel selector TUI, equivalent to using `--channel` with no argument.
- Running `tenex-edge launch` without `--channel` in a non-TTY environment fails loudly, asking for `--channel <id>`.
- When no explicit channel is provided outside of `tenex-edge launch`, the project channel is used instead of creating a session-specific room. <!-- [^8510c-a05be] -->
