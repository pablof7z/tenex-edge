---
title: Tenex-Edge Mgmt Session List
slug: tenex-edge-mgmt-session-list
topic: tenex-edge
summary: `tenex-edge mgmt session list` is an interactive TUI for local session control with fuzzy search, navigation, selection toggles, and killing
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

`tenex-edge mgmt session list` is an interactive TUI for local session control with fuzzy search, navigation, selection toggles, and killing. It replaces the old top-level `tui` command, which has been completely removed with no aliases. The TUI groups sessions by workspace/channel with a details pane and preserves PTY attach, panes, and refresh behavior from the previous TUI. <!-- [^019f5-de9f4] -->

It is backed by a canonical daemon query joining session records, memberships, lifecycle, and endpoints, replacing the unreliable client-side `agents_list_sessions` + `pty_status` merge. <!-- [^019f5-9fc39] -->

It shows all locally owned and manageable sessions; remote sessions remain visible through `tenex-edge human who`, and the local management UI does not imply it can kill them. <!-- [^019f5-3e7fa] -->

Non-PTY sessions are included without attach errors; opening a non-PTY row does not exit the TUI. <!-- [^019f5-2f8f0] -->

## Fuzzy Search

`mgmt session list` fuzzy search with `/` matches across handle, title, activity, workspace/channel, host, cwd, and transport. <!-- [^019f5-5c7ff] -->

## Navigation and Selection

`mgmt session list` supports `↑/↓` and `j/k` for row navigation. Use `Space` to toggle selection, `a` to select all visible search results, and `u` to clear the current selection. <!-- [^019f5-891e2] -->

## Killing Sessions

`mgmt session list` kills sessions with a double-`K` confirmation. The confirmation prompt shows the exact selected count and handles. It kills the selected sessions, or the current row when none are selected. <!-- [^019f5-7dd12] -->
