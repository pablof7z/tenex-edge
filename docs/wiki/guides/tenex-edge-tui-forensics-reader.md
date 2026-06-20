---
title: tenex-edge TUI Forensics Reader
slug: tenex-edge-tui-forensics-reader
topic: tenex-edge
summary: Log files are read using a tail-read helper that seeks to file_size minus 2MB and skips the partial first line, capping reads at ~2MB per refresh cycle
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-17
updated: 2026-06-17
verified: 2026-06-17
compiled-from: conversation
sources:
  - session:3b87cdd2-dc84-40d5-9bf0-677e282fe0e4
---

# tenex-edge TUI Forensics Reader

## Log File Reading

Log files are read using a tail-read helper that seeks to file_size minus 2MB and skips the partial first line, capping reads at ~2MB per refresh cycle. The hook log (hook-calls.jsonl) is read with a 20MB tail limit instead of 2MB, to ensure session-start events like user-prompt-submit are not truncated by subsequent tool-use hooks pushing them out of the tail window. <!-- [^3b87c-4] -->

## Background Loading & Input Handling

seed_live_sessions uses call_no_spawn instead of call() to avoid daemon spawn latency (up to 30s) in the TUI; if the daemon is not running, the TUI shows no live sessions rather than blocking. load_hook_tail_snapshot runs in a background thread, with the event loop picking up results non-blockingly via mpsc::try_recv(); keyboard input is processed at 100ms intervals even while a load is in flight. Filter changes (p, s, a) reset next_refresh to zero so the new snapshot loads immediately. <!-- [^3b87c-5] -->

## Pane Titles & Header Bar

Pane titles show 'developer@tenex-edge [03cfa7]' format (agent@project [short-session-id]) instead of just the short session ID, with fallbacks when only agent or project is known. The header bar shows active project filters like 'project=tenex-edge,src' when multiple projects are selected. <!-- [^3b87c-6] -->

## Project Filter Popup

Pressing 'p' opens a centered multi-select popup listing all known projects, with up/down to move cursor, space to toggle selection, [x] for selected (green) and [ ] for not selected. The project popup title shows 'Projects (2 selected)' when items are checked or 'Projects (all)' when nothing is checked. Pressing 'a' inside the project popup clears all selections. Pressing esc, enter, or 'p' closes the project popup and immediately reloads with the new filter. <!-- [^3b87c-7] -->

## Timeline Display

Each timeline line displays as '+0.0s  event-type    summary' — relative timestamp, event type in a fixed 18-char column, then a smart summary. user-prompt-submit summary shows the actual prompt text in light yellow for quick scanning. inject events show only the first line of the injected text, truncated, instead of full JSON walls. pre-tool-use/post-tool-use events show the tool name, and show the command if the tool is Bash. cmd events show just the subcommand, stripping the tenex-edge binary name. hook finished ok lines are entirely suppressed from the timeline; only errors surface. <!-- [^3b87c-8] -->

## Focus Mode & Detail Views

Focus mode (Enter/f) allows up/down navigation of timeline lines, with the selected line getting a subtle blue highlight. Focus mode title shows 'developer@tenex-edge [03cfa7] (12/34)' indicating current position in the timeline. A detail panel below the timeline in focus mode shows the full content of the selected line with proper word-wrapping and real newlines, auto-sized to content height capped at 12 lines. Scrolling down past the last line in focus mode snaps back to follow-tail mode; new events auto-advance the cursor. Pressing Enter in grid mode enters focus mode; in focus mode, up/down selects lines and Enter opens a full-screen detail overlay; f/Esc exits zoom. The detail overlay fills nearly the full terminal, shows the complete content of the selected line with proper line wrapping, labels itself with the event type in the title bar, and any key (except q) closes it. <!-- [^3b87c-9] -->

## Pane Navigation

Tab and left/right arrow keys switch panes in both grid and focus mode. <!-- [^3b87c-10] -->
