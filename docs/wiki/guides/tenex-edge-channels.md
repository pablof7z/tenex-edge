---
title: Tenex-Edge Channels
slug: tenex-edge-channels
topic: tenex-edge
summary: Agents are not removed from channels when a session ends
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-06-29
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:3c769f4a-9947-4d7b-a8f5-58355620b951
  - session:661ebf6b-e01b-4ff6-b9c7-5042b900c788
  - session:b07a57a3-67a1-4c44-a8fc-58a1bb97860a
  - session:bd8689c8-4a5f-45b3-9dbe-758baec2a2f4
  - session:b20ef4ab-0b54-4770-a549-4ed195c0035e
  - session:c55adda0-b071-4b76-9d24-a0cbcb5b6e0c
---

# Tenex-Edge Channels

## Agent Retention

Agents are not removed from channels when a session ends. <!-- [^3c769-6616d] -->


The chat_write bail requiring a concrete session codename is removed; delivery routes by pubkey alone. <!-- [^bd868-93a15] -->
## Invite

`tenex-edge invite <slug[@backend]>` spawns a fresh session for the invited agent in the current channel. It is an explicit command, not an auto-add side-effect of @-mention. <!-- [^661eb-712ca] -->

## Channels Switch

`tenex-edge channels switch` is an agent-only command. Channel paths are project-relative with no project prefix—for example, `planning` or `epic999/planning`, never `tenex-edge/planning`. When ambiguous, it returns a structured error with exit code 2 and provides copy-paste-ready command re-runs instead of an interactive prompt. <!-- [^661eb-a80c7] -->

## Channel References

Channel references use forward-slash hierarchy (e.g. `tenex-edge/planning`), never dots. <!-- [^661eb-ba480] -->

## Channel Creation

When an agent creates a channel, the daemon auto-switches the creating agent's session into the new channel, and the CLI prints `switched to it`. The auto-switch is unconditional for genuine agent sessions because the brand-new room needs none of the switch path's occupancy or membership guards. A `channels create` that hits the dedup path (name already exists) does not auto-switch, because switching into a pre-existing channel needs the occupancy checks that only `channels switch` runs. The `--agent` flag is optional on `channels create`; a channel can be created and joined without specifying any agents. When `channels create` is invoked with no `--agent` targets, no kind:9 orchestration event is published and `orchestration_event_id` comes back empty.

<!-- citations: [^b07a5-7de80] [^b20ef-a6805] -->

## Channel Awareness

Unnamed channels (channels whose name is empty or equals their own id) are excluded from the "Other active channels" awareness block. <!-- [^c55ad-57ccb] -->
