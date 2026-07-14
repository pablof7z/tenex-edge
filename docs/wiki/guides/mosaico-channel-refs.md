---
title: Mosaico Channel Reference Grammar
slug: mosaico-channel-refs
topic: mosaico
summary: Channel references use dotted paths as the single public addressing grammar
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-14
updated: 2026-07-14
verified: 2026-07-14
compiled-from: conversation
sources:
  - session:019f606f-1d25-76d1-95da-a7bb383cbe6d
---

# Mosaico Channel Reference Grammar

## Channel Reference Grammar

Channel references use dot-separated paths as the single public addressing grammar. Qualified paths include the workspace root, such as `workspace.epic.review`; shorter unique suffixes resolve within the current workspace root. Canonical channel output uses dotted paths. The `@deadbeef` opaque-ID form serves as an escape hatch for channel addressing. <!-- [^019f6-cb0fe] -->

## Rejected Forms

Channel paths reject slash syntax, empty segments, and filesystem semantics. <!-- [^019f6-e0798] -->

## Workspace and Channel Interaction

Combining `--workspace` with a workspace-qualified `--channel` path is forbidden. Qualified channel resolution selects both the fabric root and that workspace's local directory; if the target workspace has no local binding, the launch fails loudly. <!-- [^019f6-1a8a4] -->
