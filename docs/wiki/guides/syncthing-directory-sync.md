---
title: Syncthing Directory Sync
slug: syncthing-directory-sync
topic: syncthing-directory-sync
summary: The Syncthing directory syncs only markdown (.md) documents and excludes all other file types (no git, code, or build artifacts)
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-09
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:561703ff-71f3-43ce-923c-c69c735f83c5
---

# Syncthing Directory Sync

## File Type and .stignore Rules

The Syncthing directory syncs only markdown (.md) documents and excludes all other file types (no git, code, or build artifacts). The .stignore file uses a first-match-wins rule order: `!*/` to un-ignore all directories (allowing Syncthing to recurse), `!*.md` to un-ignore markdown files, and `*` to ignore everything else. <!-- [^56170-1] -->
