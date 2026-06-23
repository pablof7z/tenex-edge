---
title: Tenex-Edge Post-Tool-Use Awareness
slug: tenex-edge-post-tool-use-awareness
topic: tenex-edge
summary: The PostToolUse hook for Claude Code emits project-scoped, delta-gated awareness of sibling sessions only when something has changed since the last check, produ
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
  - session:9337d29e-ac62-417c-8e99-0cc22cbbfad3
---

# Tenex-Edge Post-Tool-Use Awareness

## PostToolUse Delta-Gated Awareness

The PostToolUse hook for Claude Code emits project-scoped, delta-gated awareness of sibling sessions only when something has changed since the last check, producing no output when nothing changed. The old `pc awareness --hook PostToolUse` hook (global, ungated, entire cross-repo roster on every tool call) is removed and replaced by the tenex-edge delta-gated hook. Awareness deltas are project-scoped (only sessions in the current project), self-excluded (never echoing the querying session's own changes), and delta-gated (a per-session cursor prevents the same fact from appearing twice). Direct inbox messages surface immediately in PostToolUse output and are not rate-limited by the debounce floor.

<!-- citations: [^a0037-9] [^a0037-15] -->
## Debounce and Cursor Mechanics

A 60-second debounce floor gates PostToolUse delta checks: the first check of a turn always fires (showing changes since turn start), and subsequent checks only fire if at least 60 seconds have elapsed since the last check. The `turn_check_due()` function returns the `since` timestamp only if ≥60 seconds have elapsed since the last check (or turn start); otherwise it returns `None` and the delta is skipped entirely. The per-session cursor (`turn_state.last_check_at`) is reset to 0 at each turn start and advances to `now` on each check that actually runs; the cursor write happens inside the daemon (single writer via daemon-mediated RPC), avoiding multiwriter risk. The schema migration for `last_check_at` uses the guarded `ALTER TABLE ... ADD COLUMN` pattern with `let _ =` to ignore errors if the column already exists, matching the established convention. The `last_check_at` column is also included in the `CREATE TABLE` statement for `turn_state` (matching the pattern used by `first_seen`), so in-memory databases get the column without relying on ALTER TABLE migrations.

<!-- citations: [^a0037-10] [^a0037-16] -->
## Delta Output Format

PostToolUse delta output includes the session's short code, title, and activity line (e.g., 'editing hooks.rs'). Session identifiers use the 6-character `session_short_code` (matching `who`/tmux display), not the raw UUID. Idle transitions are shown with '· idle' (going idle bumps the status row's `updated_at`, so they are already caught by `list_status_changes_since` and render as '<title> · idle'). The `build_status_delta` helper in `who.rs` is shared between turn-start and turn-check (PostToolUse) delta rendering, with a self-exclusion parameter to filter out the calling session.

<!-- citations: [^a0037-11] [^a0037-17] -->
## Query and Context Envelope

The delta query reuses `list_status_changes_since` and `list_new_peer_sessions`, scoped to the current project and excluding the querying session's own ID, using the same formatter as the turn-start delta. The `list_status_changes_since` query returns the activity line in addition to the title. Claude Code PostToolUse only reads context from the `hookSpecificOutput.additionalContext` envelope (`{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"..."}}`); plain stdout is ignored. An `EmitFormat` enum selects the JSON envelope only for Claude Code PostToolUse; UserPromptSubmit and opencode remain plain text; Codex stays `systemMessage`.

The opencode integration maps its mid-turn `transform` calls to the `post-tool-use` hook, injecting that hook's stdout for non-destructive mid-turn peeks (new messages + sibling deltas) instead of using bespoke shell-outs. <!-- [^9337d-4] -->

<!-- citations: [^a0037-12] [^a0037-18] -->
## Process Timeout

The PostToolUse hook process timeout is set to 10 seconds (matching the per-tool-call synchronous cadence), not 30 seconds, since it fires synchronously before tool results and the actual work is a sub-millisecond Unix-socket RPC, so it must fail fast if the daemon hangs.

<!-- citations: [^a0037-13] [^a0037-19] -->
## Implementation History

The delta-gated PostToolUse awareness feature was committed as three code commits on master: `0f5dad27` (delta-gated PostToolUse awareness), `c9a6ea06` (short-code session display fix), and `bc9b2e10` (preserved peer docs/wiki work). <!-- [^a0037-14] -->
