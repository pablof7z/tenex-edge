---
type: episode-card
date: 2026-06-16
session: 633f8f7f-37f8-409c-90a9-ef64b0dc3216
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/633f8f7f-37f8-409c-90a9-ef64b0dc3216.jsonl
salience: product
status: superseded
subjects:
  - distillation-error-logging
  - statusline-error-surfacing
supersedes: []
related_claims: []
source_lines:
  - 103-136
  - 138-139
captured_at: 2026-06-16T09:33:44Z
---

# Episode: Surface distillation failures via log files and statusline

## Prior State

LLM distillation failures were silently swallowed — no logs, no traces. `complete_via_rig` used `.ok()?`, `CommandDistiller` redirected stderr to `/dev/null`, and `distill_session` fell through to a retain-existing-title path. The only visible symptom was a frozen title and blank activity line.

## Trigger

User asked whether any logs exist on LLM failure; discovering none, the user explicitly proposed a two-part design: write errors into a log under `~/.tenex/edge/` and have the statusline read those logs to flash per-session errors.

## Decision

Adopt an error-surfacing architecture: distillation (and similar) failures will write to log files in `~/.tenex/edge/`, and the statusline will read those logs to display session-scoped error indicators to the user.

## Consequences

- Distillation code paths must replace silent `.ok()?` / `None` returns with structured error writes to `~/.tenex/edge/` log files.
- The statusline (currently pure-read, never writes to state.db) must be extended to read error logs and render a per-session error indicator.
- The invariant that statusline is peek-only is preserved (it reads logs, doesn't write state).
- Error visibility changes from 'frozen title + blank activity' to an explicit, session-attributed flash.

## Open Tail

- Log file format and rotation policy for `~/.tenex/edge/` not yet specified.
- Statusline UI contract for error flashing (icon, duration, dismissal) not yet defined.
- Whether other subsystems beyond distillation should also write to the same error log surface.

## Evidence

- transcript lines 103-136
- transcript lines 138-139

