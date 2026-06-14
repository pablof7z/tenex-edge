---
title: Tenex-Edge TUI
slug: tenex-edge-tui
topic: tenex-edge
summary: The TUI groups sessions by project, navigable via Left/Right arrow keys across project tabs
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-14
updated: 2026-06-14
verified: 2026-06-14
compiled-from: conversation
sources:
  - session:9f7f245f-0fad-4211-a86b-95ea3cbb532e
  - session:656e1e6b-2569-42da-8844-768a5e74788e
---

# Tenex-Edge TUI

## Scrolling Behavior

The TUI groups sessions by project, navigable via Left/Right arrow keys across project tabs. In the 'All' tab, session labels display in `slug@project` format so the project of each session is identifiable. The list scrolls — draw_tui renders only lines that fit the terminal height, keeps the selected row in view, and shows ↑N more above / ↓N more below indicators. Exited sessions are hidden by default and toggled visible by pressing 'e'. The 'Spawnable (no session)' label is renamed to 'Agents'. The '[spawnable via claude]' label is renamed to '[claude]'. Agents appear in all project tabs since they are cross-project.

<!-- citations: [^9f7f2-15] [^9f7f2-21] [^9f7f2-25] [^656e1-2] -->

## Project Tab Priority and Visibility

Project tabs are prioritized so that projects with live sessions appear first (alphabetically), followed by projects with recent activity within 7 days (alphabetically). Projects that have had no agents online in the past 7 days are hidden from the tab bar by default. Selecting a hidden project (>7d inactive) via fuzzy search temporarily injects it into the visible tabs until the next periodic refresh unless activity resumes. <!-- [^656e1-3] -->

## Fuzzy Search

Pressing '/' opens a fuzzy search overlay to filter and select projects by case-insensitive substring. In the overlay, Up/Down arrows navigate results, Enter jumps to the selected project tab, and Escape cancels. <!-- [^656e1-4] -->
