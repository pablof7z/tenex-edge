---
title: tenex-edge MVP Scope
slug: tenex-edge-mvp-scope
topic: tenex-edge
summary: The MVP for tenex-edge is advisory lock (collision avoidance) plus shared-bug deduplication, strictly solo, across Claude Code and Codex on one machine â no c
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-07
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
---

# tenex-edge MVP Scope

## MVP Scope

The MVP for tenex-edge is advisory lock (collision avoidance) plus shared-bug deduplication, strictly solo, across Claude Code and Codex on one machine — no cross-person features. <!-- [^8a3eb-19] -->

## Phase 0

Phase 0 needs zero Nostr: prove presence and advisory file-lock locally between two Claude Code sessions using hook shims and a local daemon with SQLite, then make Nostr the growth axis afterward. <!-- [^8a3eb-20] -->
