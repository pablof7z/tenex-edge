---
type: episode-card
date: 2026-06-16
session: 633f8f7f-37f8-409c-90a9-ef64b0dc3216
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/633f8f7f-37f8-409c-90a9-ef64b0dc3216.jsonl
salience: product
status: active
subjects:
  - distill-error-surfacing
  - statusline-error-display
  - session-errors-table
supersedes:
  - 2026-06-16-1-surface-distillation-failures-via-log-files
related_claims: []
source_lines:
  - 130-292
captured_at: 2026-06-16T09:36:20Z
---

# Episode: Surface distillation errors via log file and statusline flash

## Prior State

Distillation failures were silently swallowed — `.ok()?` and `None` returns throughout distill.rs with no logging, no DB record, and no user-visible indication. If the LLM was down or the API key wrong, the only symptom was a frozen title and blank activity line.

## Trigger

User discovered the silent-failure behavior and explicitly directed: 'ideally we have a way of surfacing this kind of errors, by writing into a log in ~/.tenex/edge/ and also by having the statusline read logs so it can flash an error in the right session that generated the issue'

## Decision

Adopt two-pronged error surfacing: (1) append-only log at `~/.tenex/edge/logs/<session_short_code>.log` with timestamped `[distill] ERROR:` lines; (2) new `session_errors` table in state.db so the hot-path statusline RPC can read recent errors and render `⚠ <msg>` in red per session. distill.rs return type will change from `Option<SessionLabels>` to `(Option<SessionLabels>, Option<String>)` to carry error info out rather than swallowing it, keeping distill.rs side-effect-free.

## Consequences

- distill.rs signature change — callers in runtime.rs must handle the new error field
- state.rs gains session_errors table + two accessor methods
- rpc_statusline must read recent errors and add an `error` field to StatuslineView
- render_statusline gains `⚠ <msg>` red-flash rendering for the owning session
- runtime.rs writes to both log file and DB on distillation failure
- distill.rs stays pure (no side effects); error propagation is the caller's responsibility

## Open Tail

- Implementation not yet started — user was asked 'Want me to implement this?' with no response recorded
- Retention/rotation policy for session_errors rows and log files not yet specified
- Exact schema of session_errors table not yet defined

## Evidence

- transcript lines 130-292

