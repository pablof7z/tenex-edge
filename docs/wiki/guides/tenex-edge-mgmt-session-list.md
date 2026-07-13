---
title: Tenex-Edge Mgmt Session List
slug: tenex-edge-mgmt-session-list
topic: tenex-edge
summary: `tenex-edge mgmt session list` is an interactive, inline Inquirer-style picker for local session control
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-13
updated: 2026-07-13
verified: 2026-07-13
compiled-from: conversation
sources:
  - session:019f5a74-0a91-7340-8299-8ac3dccfa36d
---

# Tenex-Edge Mgmt Session List

## Overview

`tenex-edge mgmt session list` is an interactive, inline Inquirer-style picker for local session control. It stays in the normal terminal flow as a compact filterable checklist rather than a full alternate-screen TUI application: the user filters sessions, toggles selection, and kills the selection, then the picker exits cleanly. (Previously: a full alternate-screen TUI with panes, a details pane, and PTY attachment preserved from the prior top-level `tui`.)

It replaces the old top-level `tui` command, which has been completely removed with no aliases. The alternate-screen/pane session-TUI layer (~1,500 lines) has been deleted and replaced with this compact inline picker. Top-level `tenex-edge agents` is also deleted completely with no aliases.

It is backed by a canonical daemon query joining session records, memberships, lifecycle, and endpoints, replacing the unreliable client-side `agents_list_sessions` + `pty_status` merge. The `operator_sessions` projection is the daemon-owned view that provides this canonical view of all local sessions.

It shows all locally owned and manageable sessions; remote sessions remain visible through `tenex-edge human who`, and the local management UI does not imply it can kill them. Non-PTY sessions are included without attach errors.

<!-- citations: [^019f5-de9f4] [^019f5-9fc39] [^019f5-3e7fa] [^019f5-2f8f0] [^019f5-f32f3] -->
## Fuzzy Search

`mgmt session list` fuzzy search with `/` matches across handle, title, activity, workspace/channel, host, cwd, and transport. The inline picker supports type-to-filter in the normal terminal flow.

<!-- citations: [^019f5-5c7ff] [^019f5-0df15] -->
## Navigation and Selection

`mgmt session list` supports `↑/↓` and `j/k` for row navigation. Use `Space` to toggle selection, `a` to select all visible search results, and `u` to clear the current selection. `Enter` submits the picker.

<!-- citations: [^019f5-891e2] [^019f5-905b0] -->
## Killing Sessions

`mgmt session list` kills sessions with a double-`K` confirmation. The confirmation prompt shows the exact selected count and handles, and defaults to No. It kills the selected sessions, or the current row when none are selected. Cancelling the inline picker with Esc restores the terminal to normal mode without performing any kill.

<!-- citations: [^019f5-7dd12] [^019f5-3e094] -->
