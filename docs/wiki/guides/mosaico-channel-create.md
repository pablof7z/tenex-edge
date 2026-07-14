---
title: Mosaico Channel Create
slug: mosaico-channel-create
topic: mosaico
summary: "`mosaico channel create` creates a nested channel from a dotted path and focuses it"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-06-29
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:b20ef4ab-0b54-4770-a549-4ed195c0035e
  - session:76eb2229-ffbb-4b24-84b9-7d9caabc93ca
---

# Mosaico Channel Create

## Parent Channel Resolution

`mosaico channel create <path> --about <description>` creates a channel beneath the caller's current channel. Parent segments in `<path>` identify the parent and the final segment names the new channel. The command accepts repeatable `--agent <slug@backend>` targets and an optional `--session <identity>` resolution anchor.

<!-- citations: [^b20ef-fa36b] [^b20ef-750ad] [^b20ef-083b2] -->
## Auto-Switch on Create

The `--agent` flag is optional. When an agent creates a channel, the session is automatically switched into it. When no `--agent` is provided, the channel is created and joined but no kind:9 orchestration event is published, and `orchestration_event_id` comes back empty. The auto-switch uses a shared `rehome_session_to_channel` helper so create and switch share the same route-scope and identity move logic. The CLI prints `switched to it` after auto-switching into a newly created channel.

<!-- citations: [^b20ef-a739c] [^b20ef-96613] -->

## Duplicate Channel Handling

`mosaico channel create` on an existing channel name errors out instead of silently deduping. The error identifies the existing channel and points to `mosaico channel switch <name>`. It propagates through the RPC layer and exits non-zero. <!-- [^b20ef-5f557] -->

## Default Channel

`mosaico launch` with no `--channel` argument defaults to the project root channel. Passing `--channel ""` explicitly opens the interactive channel picker. <!-- [^76eb2-2c62e] -->
