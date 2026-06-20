---
title: tenex-edge Thread Protocol
slug: tenex-edge-thread-protocol
topic: tenex-edge
summary: "When an agent finishes producing text (stop hook), it must publish a kind:1 TurnReply with its own key, e-tagging the root event and the prompt that triggered t"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-10
updated: 2026-06-12
verified: 2026-06-10
compiled-from: conversation
sources:
  - session:40a4d401-2520-4781-b747-b0ef19594bed
  - session:cd74a605-9f83-4e21-a885-4d900e88ce07
---

# tenex-edge Thread Protocol

## Thread Protocol

When an agent finishes producing text (stop hook), it must publish a kind:1 TurnReply with its own key, e-tagging the root event and the prompt that triggered the turn. Thread e-tags must use NIP-10 markers: `["e", root_event_id, "", "root"]` and `["e", reply_event_id, "", "reply"]`. The first user prompt in a session is the thread root (no e-tags); subsequent user prompts must carry `root` and `reply` e-tags linking to the thread root and the last agent TurnReply respectively. Each session tracks two thread IDs: `thread_root_event_id` (the first user prompt, immutable once set) and `last_prompt_event_id` (updated on every user prompt). The TurnReply event ID must be persisted after publishing so subsequent user prompts can reference it as the `reply` e-tag. <!-- [^40a4d-3] -->

Thread IDs must be captured atomically at the start of `rpc_turn_end` before any async polling, preventing a concurrent `user_prompt` from overwriting `last_prompt_event_id` and corrupting the reply tag. <!-- [^40a4d-4] -->

TurnReply publishing must poll the transcript at `turn_end` for up to ~2 seconds, waiting for content that differs from a baseline snapshot taken at `turn_start`, to handle the stop hook firing before the transcript is written. <!-- [^40a4d-5] -->

A reply via `inbox reply --id` creates a NIP-29 kind:1 event that e-tags the original event and p-tags the sender. <!-- [^cd74a-9] -->
