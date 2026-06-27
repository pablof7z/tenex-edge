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
updated: 2026-06-26
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:162f9965-82ca-420b-aa24-99faa15cb59a
  - session:8510c3cc-9722-47a4-90ee-2f489646f5b8
---

# Tenex-Edge Channels

## Architecture Constraints

The channel server must be a thin stream-consumer that never independently writes state.db, avoiding re-introduction of multi-writer corruption. <!-- [^162f9-24] -->

wait-for-mention stays as the portable floor for all harnesses; the channel adapter is a Claude-specific ceiling. <!-- [^162f9-25] -->

All three hosts (Claude Code, Codex, OpenCode) have idle-wake primitives: Claude uses channels (MCP push), Codex uses app-server turn/start (JSON-RPC), and OpenCode uses POST /session/{id}/prompt_async (local HTTP). <!-- [^162f9-26] -->

## Channel Creation

Channel creation logic is unified into a single `ensure_channel_ready` primitive. Both `ensure_session_room` and `rpc_channels_create` call this primitive as thin wrappers, differing only in the channel name source (auto-generated vs operator-chosen). The primitive includes an admin-reflection poll to ensure a freshly-created group's dependent grants apply on the first try. <!-- [^8510c-ad6e8] -->
