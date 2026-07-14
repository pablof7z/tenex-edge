---
title: Tenex-Edge Channel Reference Grammar
slug: tenex-edge-channel-refs
topic: tenex-edge
summary: Channel references use slash-separated paths as the single public addressing grammar, replacing dotted refs entirely with no compatibility aliases
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

# Tenex-Edge Channel Reference Grammar

## Channel Reference Grammar

Channel references use slash-separated paths as the single public addressing grammar, replacing dotted refs entirely with no compatibility aliases. Absolute channel paths have the form `/workspace/channel1/channel2`; relative channel paths are resolved relative to the current workspace root, never the active channel. Canonical channel output is always an absolute path of the form `/workspace/channel`. The `@deadbeef` opaque-ID form serves as an escape hatch for channel addressing. <!-- [^019f6-cb0fe] -->

## Rejected Forms

Dotted channel references are rejected outright with no compatibility aliases, parser aliases, or fallback support. Channel paths reject filesystem semantics: `.`, `..`, `//`, and trailing `/` are not permitted. <!-- [^019f6-e0798] -->

## Workspace and Channel Interaction

Combining `--workspace` with an absolute `--channel` path is forbidden. Absolute channel path resolution must select both the fabric root and that workspace's local directory; if the target workspace has no local binding, the launch fails loudly. <!-- [^019f6-1a8a4] -->

## Migration Impact

The slash-path channel grammar migration requires no relay or SQLite identity migration because Nostr and SQLite storage use opaque channel IDs plus parent IDs rather than human-readable channel strings. <!-- [^019f6-45c8d] -->
