---
type: episode-card
date: 2026-07-09
session: a62822c5-d09c-4a83-9251-a3856d276ac4
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a62822c5-d09c-4a83-9251-a3856d276ac4.jsonl
salience: product
status: active
subjects:
  - distill-failure-surfacing
  - session-title
  - agent-warnings
  - ollama-config
supersedes:
  - 2026-07-09-a62822c5d09c-fbf565a0-4-persistent-status-generation-failures-now-surfaced
related_claims: []
source_lines:
  - 1117-1118
  - 1120-1232
  - 1262-1262
  - 1405-1449
  - 1699-1938
captured_at: 2026-07-09T20:52:45Z
---

# Episode: Persistent distill failures now surface as throttled agent-facing warnings

## Prior State

Distill failures were completely invisible: the sole handler was a debug `slog` line in a per-session log file (`runtime.rs:281-283`). No persistence, no counting, no warning to agents or users. The statusline had a dead `distill_error` field (always `None`/`null`). A previous `session_errors` table and `record_session_error`/`get_recent_session_error` existed but were deleted in a schema rewrite (commit `fbb50ee6`) with no replacement. Session titles would silently remain empty forever when the distiller couldn't reach its LLM endpoint.

## Trigger

User discovered session titles were persistently empty (line 1120). Initial theory was a debounce/timing issue, but live investigation revealed the actual root cause: `OLLAMA_HOST` in `~/.zshrc` and `providers.json` both pointed at `localhost:8081` (squatted by Docker Desktop), while Ollama was actually running on port 11434. Every distill call silently failed and fell back to `current_title=None`. User then directed: persistent status-generation failures should be surfaced to agents (using non-internal language, not the word 'distillation'), throttled to a few times per hour so agents don't pester the user (line 1262).

## Decision

Added `distill_fail_streak` and `distill_notice_at` columns to `sessions` table via additive idempotent migration (no wipe/rebuild). Added `record_distill_failure`/`mark_distill_notice` Store methods. Wired the failure branch in `runtime.rs` observe loop to record failures. `assemble_turn_start` now injects a `<warnings>` entry — *“This session's status/title updates haven't been generating successfully for a while. You may want to let the user know status updates aren't working right now.”* — after 3 consecutive failures, throttled to at most once per 15 minutes while the streak persists. A successful distill resets the streak to 0. No internal terminology used in agent-facing text. Also fixed Ollama config: updated `OLLAMA_HOST` in `~/.zshrc` and `providers.json` to `http://localhost:11434`, started Ollama via `brew services`.

## Consequences

- New persistent state: `distill_fail_streak` and `distill_notice_at` columns on `sessions` table with idempotent `ensure_columns` migration mirroring existing `outbox_backoff` pattern
- Agents now receive actionable warnings about status-generation failures and can alert the user, closing the silent-failure gap
- Throttling (3-failure threshold + 15-min cooldown) prevents agent pestering while issue persists
- Success resets streak, so warnings stop automatically after recovery
- Dead statusline `distill_error` field remains orthogonal and unused — not repurposed
- Ollama config fix confirmed live: next distill attempt succeeded with a real title
- Code compiled and 3 regression tests pass (below-threshold silence, first-fire + throttle suppression, streak-reset-on-success), but daemon restart still needed to deploy to running sessions

## Open Tail

- Daemon restart pending — new distill-notice code is compiled into `target/debug` but the live daemon managing all sessions is a separate running process that hasn't been restarted yet
- The dead `distill_error` statusline field (always null) was not cleaned up — it remains as vestigial code

## Evidence

- transcript lines 1117-1118
- transcript lines 1120-1232
- transcript lines 1262-1262
- transcript lines 1405-1449
- transcript lines 1699-1938

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-09-a62822c5d09c-fbf565a0-4-persistent-distill-failures-now-surface-as.json`](transcripts/2026-07-09-a62822c5d09c-fbf565a0-4-persistent-distill-failures-now-surface-as.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-09-a62822c5d09c-fbf565a0-4-persistent-distill-failures-now-surface-as.json`](transcripts/raw/2026-07-09-a62822c5d09c-fbf565a0-4-persistent-distill-failures-now-surface-as.json)
