---
title: Tenex-Edge Channels
slug: tenex-edge-channels
topic: tenex-edge
summary: The channel server must be a thin stream-consumer that never independently writes state.db, avoiding re-introduction of multi-writer corruption.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-09
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:162f9965-82ca-420b-aa24-99faa15cb59a
---

# Tenex-Edge Channels

## Architecture Constraints

The channel server must be a thin stream-consumer that never independently writes state.db, avoiding re-introduction of multi-writer corruption. <!-- [^162f9-24] -->

wait-for-mention stays as the portable floor for all harnesses; the channel adapter is a Claude-specific ceiling. <!-- [^162f9-25] -->

All three hosts (Claude Code, Codex, OpenCode) have idle-wake primitives: Claude uses channels (MCP push), Codex uses app-server turn/start (JSON-RPC), and OpenCode uses POST /session/{id}/prompt_async (local HTTP). <!-- [^162f9-26] -->
