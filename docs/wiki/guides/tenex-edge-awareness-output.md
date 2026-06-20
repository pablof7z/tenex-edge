---
title: tenex-edge Awareness Output
slug: tenex-edge-awareness-output
topic: tenex-edge
summary: PostToolUse awareness output must be delta-gated, project-scoped, and self-excluded â only emitting sibling session changes in the current project since the l
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-15
updated: 2026-06-16
verified: 2026-06-15
compiled-from: conversation
sources:
  - session:a0037729-ad51-460a-880d-0a9699f6ee41
  - session:rollout-2026-06-09T12-58-38-019eabd1-dde2-76c2-84e3-9edc3e78e48f
  - session:rollout-2026-06-16T17-43-45-019ed0e3-68e5-7091-899d-6a4e0fcb5716
---

# tenex-edge Awareness Output

## PostToolUse Awareness Output

PostToolUse awareness output must be delta-gated, project-scoped, and self-excluded — only emitting sibling session changes in the current project since the last check, never echoing the agent's own session. The 'changes since your last turn' injection is scoped to the current session's project, preventing cross-project status updates from leaking into unrelated sessions. When no sibling changes have occurred since the last check, PostToolUse must produce zero output (complete silence). The delta query reuses `list_status_changes_since` + `list_new_peer_sessions`, scoped to `rec.project` and excluding `rec.session_id`, using the same formatter as the turn-start delta. Store::list_new_peer_sessions and Store::list_status_changes_since accept an optional project filter for scoped delta reads, and turn_start passes the current session's rec.project into the peer/status delta queries to scope them. The `list_status_changes_since` query must return the activity field in addition to the title, so deltas can render `<title> — <activity>` for active sessions and `<title> · idle` for idle ones. PostToolUse awareness must include the activity line (not just title) in deltas, and must show idle transitions. Regression tests exist that insert same-time deltas in two projects and assert only the current project's deltas are returned. Direct inbox messages in PostToolUse awareness are not rate-limited and surface immediately. Delta semantics are preserved: full roster on first turn, appeared/changed/gone deltas after, project-scoped, self-excluded, and turn-check is a pure read. Turn-start roster and deltas are not self-excluded.

<!-- citations: [^a0037-1] [^rollo-19] [^rollo-80] -->
## Debounce and Cursor State

PostToolUse awareness must be debounced with a 60-second floor — a check only runs if ≥60 seconds have elapsed since the last check (or since turn start for the first check of a turn). The PostToolUse cursor (`turn_state.last_check_at`) resets to the turn-start timestamp at each turn boundary, and each check that runs advances it to `now` inside the daemon (single-writer, no multiwriter risk). <!-- [^a0037-2] -->

## Hook Timeout Configuration

The PostToolUse hook timeout in settings.json must be 10 seconds (matching the per-tool-call cadence), not 30 seconds. <!-- [^a0037-3] -->
