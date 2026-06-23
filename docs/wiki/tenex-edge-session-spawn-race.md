---
title: Tenex-Edge Session Spawn Race Condition
slug: tenex-edge-session-spawn-race
topic: tenex-edge
summary: "A race condition in `spawn_session` allows two runtime tasks to be alive for the same `session_id`, causing the published kind:30315 event to flip-flop between"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-16
updated: 2026-06-16
verified: 2026-06-16
compiled-from: conversation
sources:
  - session:1b868736-ed6b-4f88-84d9-26bb320accfd
---

# Tenex-Edge Session Spawn Race Condition

## Race condition in spawn_session

A race condition in `spawn_session` allows two runtime tasks to be alive for the same `session_id`, causing the published kind:30315 event to flip-flop between two different titles on alternating heartbeats. The race occurs because the 'already running?' check (`server.rs:636`) and the actual map insert (`server.rs:2791`) are separated by `.await` points (`open_project`, `ensure_subscription`); two near-simultaneous `SessionStart` RPCs for the same `session_id` both pass the check and spawn, and the second insert evicts the first runtime from the sessions map making it an un-cancellable zombie. <!-- [^1b868-1] -->

## Fix: atomic check-and-reserve

`spawn_session` must atomically check-and-reserve an entry in `state.sessions` under a single lock before any `.await`, preventing concurrent `SessionStart` RPCs from both passing the guard and orphaning the first runtime. This atomic reserve is preserved as a belt-and-suspenders guard even after the session aggregate refactor. If subscription setup fails, the reservation must be rolled back, so a second spawn for a live `session_id` returns early. Additionally, the kind:30315 event never expires (commit 5e7a34d1), so when a stale sibling session is killed, its relay event remains forever.

<!-- citations: [^1b868-2] [^1b868-8] [^1b868-22] [^1b868-34] [^1b868-40] -->
