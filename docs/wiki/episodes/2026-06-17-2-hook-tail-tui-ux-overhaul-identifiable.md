---
type: episode-card
date: 2026-06-17
session: 3b87cdd2-dc84-40d5-9bf0-677e282fe0e4
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/3b87cdd2-dc84-40d5-9bf0-677e282fe0e4.jsonl
salience: product
status: active
subjects:
  - hook-tail-tui
  - debug-event-display
  - pane-titles
  - project-filter-popup
supersedes: []
related_claims: []
source_lines:
  - 207-217
  - 219-367
  - 368-597
captured_at: 2026-06-18T00:53:59Z
---

# Episode: Hook-tail TUI UX overhaul: identifiable panes, smart event timeline, detail overlay

## Prior State

Panes showed only short session hashes (e.g. '03cfa7'). Project filter was a single --project CLI flag. Event lines displayed raw text like 'hook ok hook finished ok' with no timestamps, no colorization, and no way to inspect details — inject events showed full JSON walls.

## Trigger

User reported panes were unidentifiable, output was illegible ('I don't know what hook ok hook finished ok even means'), wanted relative timestamps, selectable lines with a detail panel, and a multi-project toggle popup.

## Decision

Pane titles now show agent@project [short-id]. Project filter changed to multi-select popup (p key, space to toggle, comma-separated in header). Event timeline uses smart summaries: user-prompt-submit shows prompt text in yellow, inject shows truncated first line, pre/post-tool-use shows tool name, hook-finished-ok is suppressed. Relative timestamps (+0.0s format). Focus mode with line cursor. Enter opens a full-screen detail overlay with proper word-wrapping. Any key dismisses the overlay.

## Consequences

- Hook-tail is scannable — key events stand out visually without walls of text
- Multiple projects can be filtered simultaneously from within the TUI
- Detail overlay preserves the TUI context rather than replacing it
- CLI --projects flag (plural, Vec) replaces the old --project singular flag

## Open Tail

*(none)*

## Evidence

- transcript lines 207-217
- transcript lines 219-367
- transcript lines 368-597

