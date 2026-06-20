---
title: tenex-off TUI Client
slug: tenex-off-tui-client
topic: tenex-edge
summary: The tenex-off codex/ratatui-tui-client worktree (374 lines of TUI markdown table rendering) was committed and merged into master via --no-ff.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-13
updated: 2026-06-17
verified: 2026-06-13
compiled-from: conversation
sources:
  - session:74fce09f-02b4-496f-a5e1-52d19ef9fbcd
  - session:bb7ee4ef-16bf-41b9-8e75-ed6b23f0f3a4
  - session:9f7f245f-0fad-4211-a86b-95ea3cbb532e
  - session:215d979a-a054-4e2b-b349-851e0d874d6d
  - session:9bab94a2-f76f-4eda-ae41-8a6ec29ce7cf
  - session:rollout-2026-06-17T10-15-05-019ed46f-0289-7cf3-ae87-5a65210ee266
---

# tenex-off TUI Client

## TUI Client Merge

The tenex-off codex/ratatui-tui-client worktree (374 lines of TUI markdown table rendering) was committed and merged into master via --no-ff. <!-- [^74fce-16] -->

Non-attachable sessions (those without a registered tmux endpoint) are shown dimmed with a [no tmux] label, and pressing Enter on them displays a message instead of attempting attachment. After a successful spawn, the TUI exits and automatically switches to the new pane. <!-- [^bb7ee-5] -->

The TUI list renders within a scrolling viewport that fits the terminal height, keeping the selected row visible and showing `↑N more above` / `↓N more below` indicators. <!-- [^9f7f2-7] -->

Attaching from the TUI suspends the TUI, runs `tmux attach-session` as a blocking child process, and resumes the TUI when the user detaches (Ctrl-b d), rather than replacing the TUI process. <!-- [^9f7f2-8] -->

The tmux TUI exited-sessions panel defaults to a 4-hour time window and allows the user to adjust the hours filter with `+`/`-` keys using stepped increments (+1h up to 12h, +6h up to 48h, +24h beyond, minimum 1h). <!-- [^215d9-6] -->

The TUI uses ratatui for double-buffered, widget-based rendering (replacing the previous crossterm full-clear redraw approach). <!-- [^9bab9-9] -->

The `Cargo.toml` depends on `ratatui` version `0.30.1` with `crossterm_0_28` feature (and `macros`) to match the existing crossterm dependency. <!-- [^9bab9-10] -->

Agent-facing captured hook output (piped `who` and turn-start fabric injection) renders as markdown headings plus a table, not plain text. <!-- [^rollo-92] -->
