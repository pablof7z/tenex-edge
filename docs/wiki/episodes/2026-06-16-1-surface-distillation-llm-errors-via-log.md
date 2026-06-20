---
type: episode-card
date: 2026-06-16
session: 633f8f7f-37f8-409c-90a9-ef64b0dc3216
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/633f8f7f-37f8-409c-90a9-ef64b0dc3216.jsonl
salience: product
status: active
subjects:
  - distill-error-surfacing
  - statusline
  - session-errors
supersedes: []
related_claims: []
source_lines:
  - 130-136
  - 138-139
  - 271-292
  - 755-766
captured_at: 2026-06-18T00:38:26Z
---

# Episode: Surface distillation LLM errors via log file and statusline flash

## Prior State

LLM distillation failures (API errors, auth failures, timeouts) were silently swallowed via `.ok()?` and `None` returns throughout `distill.rs`. No logs were written. The only user-visible symptom was a frozen title and blank activity line — nothing was logged, nothing was surfaced.

## Trigger

User investigated how the 30315 title is generated, discovered the silent `.ok()?` / `None` error suppression, and explicitly requested: "ideally we have a way of surfacing this kind of errors, by writing into a log in ~/.tenex/edge/ and also by having the statusline read logs so it can flash an error in the right session that generated the issue"

## Decision

Implement a two-channel error surfacing system: (1) append timestamped error lines to `~/.tenex/edge/logs/<short>.log`, (2) upsert errors into a new `session_errors` table in state.db, read by `rpc_statusline` with a 5-minute TTL, rendered in the statusline as `⚠ distill: <message>` in bold red. The `distill_session` return type changed from `Option<SessionLabels>` to `(Option<SessionLabels>, Option<String>)` so actual rig error messages propagate instead of being discarded.

## Consequences

- distill.rs contract change: callers must handle the error tuple; `complete_via_rig` now returns `Result<Option<String>, String>` instead of `Option<String>`
- state.rs gained `session_errors` table + `record_session_error` / `get_recent_session_error` methods, with ALTER TABLE migration guard
- runtime.rs writes to both log file and DB when a distill task finishes with an error
- daemon/server.rs `rpc_statusline` reads errors newer than 300 seconds and includes them as `distill_error` field
- statusline renders a red `⚠ distill: <msg>` segment between status and inbox when an error is present
- All 202 lib tests pass including the updated distill tests; changes were re-applied after a parallel session-state rearchitecture overwrote the working tree

## Open Tail

*(none)*

## Evidence

- transcript lines 130-136
- transcript lines 138-139
- transcript lines 271-292
- transcript lines 755-766

