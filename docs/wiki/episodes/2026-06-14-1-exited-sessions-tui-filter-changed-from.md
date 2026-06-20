---
type: episode-card
date: 2026-06-14
session: 215d979a-a054-4e2b-b349-851e0d874d6d
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/215d979a-a054-4e2b-b349-851e0d874d6d.jsonl
salience: product
status: active
subjects:
  - tmux-tui-exited-filter
  - session-visibility
supersedes:
  - 2026-06-14-2-exited-sessions-hidden-by-default-in
related_claims: []
source_lines:
  - 1-136
captured_at: 2026-06-18T00:30:05Z
---

# Episode: Exited sessions TUI filter changed from boolean toggle to configurable hours window

## Prior State

Exited sessions panel used a simple `show_exited: bool` flag — sessions were either shown or hidden with no time-based filtering.

## Trigger

User requested: 'allow me to see past sessions from the past x hours and allow me to easily change the number of hours of the filter, default it to 4 hours'

## Decision

Replaced `show_exited: bool` with `exited_hours: Option<u64>` (None = hidden, Some(h) = show sessions within h hours). Default 4h. Added `[e]` toggle and `[-]`/`[+]` stepped adjustment keys (+1h up to 12h, +6h up to 48h, +24h beyond; reverse steps for decrease, min 1h). Section header shows active window e.g. 'Exited sessions (past 4h)'.

## Consequences

- filter_resumable now accepts Option<u64> and computes a cutoff timestamp instead of a simple boolean
- draw_tui signature changed to accept exited_hours parameter
- TUI help line dynamically shows current hours: '[e] hide exited  [-/+] 4h'
- Stepped increments prevent tedious single-hour scrolling to reach large windows

## Open Tail

*(none)*

## Evidence

- transcript lines 1-136

