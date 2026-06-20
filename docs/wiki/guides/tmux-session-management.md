---
title: Tmux Session Management
slug: tmux-session-management
topic: tenex-edge
summary: Inside tmux, switching to a pane in another window uses `switch-client -t <pane_id>` (not `select-pane`, which only works within the current window)
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-14
updated: 2026-06-17
verified: 2026-06-14
compiled-from: conversation
sources:
  - session:bb7ee4ef-16bf-41b9-8e75-ed6b23f0f3a4
  - session:656e1e6b-2569-42da-8844-768a5e74788e
  - session:9c78b46a-3169-42eb-84c1-228a6c2f6589
  - session:a7c75cc2-efc0-47db-aa7d-9332d6c63310
  - session:d8d132f9-8a71-4af0-846c-44a4a9e01dc5
---

# Tmux Session Management

## Pane and Session Targeting

Inside tmux, switching to a pane in another window uses `switch-client -t <pane_id>` (not `select-pane`, which only works within the current window). Outside tmux, attaching to or spawning a session execs `tmux attach-session -t session:window`, replacing the shell process rather than attempting select-pane. When a tmux pane is not attachable, the TUI transparently resumes the session instead of surfacing an error. Internally, the attach operation is best-effort: if attaching to a specific pane fails, the TUI falls back to resuming the session via the daemon and attaching to the fresh pane; the error only surfaces if the resume itself also fails. Pressing Enter on an attachable session with no live pane resumes the session directly instead of showing a 'Session pane not found' error. This fallback mechanism is supported by the PendingAttach struct, which carries both the pane and an optional resume_sid (session id for fallback), with freshly spawned panes having a resume_sid of None. Command arrays are passed directly to tmux without a shell, so `~` in paths is not expanded; absolute paths must be used instead.

<!-- citations: [^bb7ee-6] [^a7c75-1] [^d8d13-2] -->
## Project Tabs and Ordering

tenex-edge tmux groups sessions by project into separate tabs, with an 'All' tab as default that shows slug@project labels to identify each session's project. Project tabs are ordered with projects that have live sessions first (alphabetically), then recently-active projects within 7 days (alphabetically), recomputed on every 2-second refresh. Projects with no agent activity in the past 7 days are hidden from the tab bar by default and only surface via the '/' search. Selecting a hidden (>7d inactive) project via fuzzy search temporarily injects it into the visible tabs for the session, and it re-hides on the next periodic refresh unless activity resumes. Spawning a new tmux session from a project tab creates the session in that project tab's directory, not in the TUI process's current working directory; spawning from the 'All' tab (where there is no project filter) falls back to resolving the project from the current working directory.

<!-- citations: [^656e1-3] [^9c78b-1] -->
## Navigation and Filtering

Exited sessions are hidden by default in the tmux TUI and can be toggled visible by pressing 'e'; pressing 'e' again hides them. Pressing '/' opens a fuzzy search overlay to filter and select project tabs by substring (case-insensitive), with Up/Down to move, Enter to jump to that project tab, and Escape to cancel. <!-- [^656e1-4] -->
