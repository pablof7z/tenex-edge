---
type: episode-card
date: 2026-06-15
session: 9bab94a2-f76f-4eda-ae41-8a6ec29ce7cf
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9bab94a2-f76f-4eda-ae41-8a6ec29ce7cf.jsonl
salience: architecture
status: active
subjects:
  - tenex-edge-tui
  - ratatui-migration
supersedes: []
related_claims: []
source_lines:
  - 1-82
  - 115-124
  - 193-215
  - 297-536
captured_at: 2026-06-18T00:33:35Z
---

# Episode: TUI rendering migrated from crossterm full-clear to ratatui

## Prior State

The tmux TUI (`draw_tui`, `draw_search`) used crossterm directly with manual full-clear redraw — building a `Vec<String>`, calling `Clear(ClearType::All)`, and repainting from scratch every frame. No widget tree, no cell diffing, no double-buffer, causing potential flash/flicker on rapid repaints.

## Trigger

User asked whether the TUI was a proper ratatui app or just re-rendering everything; inspection confirmed the crossterm-only approach. User explicitly directed: 'launch a sonnet agent to update it to ratatui'.

## Decision

Replaced `draw_tui()` (~200 lines) and `draw_search()` (~75 lines) with ratatui's `Terminal<CrosstermBackend>` and `render_main()`/`render_search()` widget functions. Added `ratatui = "0.30.1"` with `crossterm_0_28` feature. Style helpers (`style_bold`, `style_cyan`, etc.) replace `owo_colors` calls. `TuiTerminal::resume()` now calls `ratatui_term.clear()` to invalidate the double-buffer after attach returns.

## Consequences

- Eliminates full-clear flicker — ratatui's diffing only repaints dirty cells
- All existing behavior preserved: tab nav, fuzzy search, attach/resume/spawn, polling refresh
- Carried forward master's '[no tmux]' tag removal and 'enter-to-spawn' changes through rebase
- Sidebar renderer later added must also use ratatui (not crossterm raw), enforcing architectural consistency

## Open Tail

*(none)*

## Evidence

- transcript lines 1-82
- transcript lines 115-124
- transcript lines 193-215
- transcript lines 297-536

