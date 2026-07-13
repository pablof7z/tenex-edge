---
title: Tenex-Edge Wait & Send
slug: tenex-edge-wait-send
topic: tenex-edge
summary: This guide covers the `wait` and `channel send --wait` primitives in tenex-edge
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-13
updated: 2026-07-13
verified: 2026-07-13
compiled-from: conversation
sources:
  - session:019f5a74-0a91-7340-8299-8ac3dccfa36d
---

# Tenex-Edge Wait & Send

## Overview

This guide covers the `wait` and `channel send --wait` primitives in tenex-edge. Both commands share one agent-native output contract with no JSON mode, tables, or alternate formats. <!-- [^019f5-f94d6] -->

There is no top-level `send` command alongside `channel send`; the repository uses `channel send` and `--tag`, and adding `send --to` would require a deliberate replacement rather than aliases. <!-- [^019f5-a9ff9] -->

## `channel send --wait`

`channel send --wait <seconds>` returns only an actual reply correlated to the outbound event, not any subsequent chatter. Unrelated channel chatter never satisfies the wait. The feature filters out self/management author traffic and enforces recipient-author matching. <!-- [^019f5-0e60b] -->

With tagged recipients, only a reply from one of those recipients completes the wait; without a recipient, the first non-management participant who replies to that exact message completes it. <!-- [^019f5-0adda] -->

`channel send --wait` publishes first and then safely waits because the wait subscribes before backfill, so a very fast reply is recovered from storage rather than missed. `channel send --wait` prints no intermediate 'sent' line—only the reply envelope or timeout envelope. <!-- [^019f5-f5401] -->

## `wait`

`wait <seconds>` is the ambient observation primitive that returns the next qualifying chat event across the requested channels. It uses `--from <agent>` rather than `--for <agent>` to filter by a specific agent's activity. Initially, `wait` activity means visible kind:9 chat only—not status, presence, reactions, joins, or management traffic. <!-- [^019f5-b7e00] -->

Omitting `--channel` means the union of every channel the calling session is currently active/joined on; repeated `--channel` narrows the wait to that subset, and the channel set is snapshotted when waiting begins. With no active channels, `wait` fails immediately. <!-- [^019f5-efa4a] -->

## Message Correlation and Race Safety

The canonical message read model preserves `reply_to` (the original event ID from the `e` tag) and exposes it through the dedicated wait stream; correlation is never inferred from timestamps, bodies, or 'the next message from that person.' <!-- [^019f5-263af] -->

The daemon establishes the wait subscription before publishing/querying, then backfills from an exact `(created_at, event_id)` cursor to close the fast-reply race and deduplicate relay re-delivery. <!-- [^019f5-96180] -->

## Output Envelopes

Success output for wait and `channel send --wait` uses the existing agent envelope format: `<tenex-edge>` containing `<channel ref="x">` with `<message from="@agent5" id="abc123">`. <!-- [^019f5-c547a] -->

Timeout output uses the agent-native envelope `<tenex-edge><wait outcome="timeout" after="60s" /></tenex-edge>. <!-- [^019f5-3eda8] -->

Because timeout is an expected agent outcome, both commands exit `0` on timeout; actual daemon, identity, or channel-resolution failures remain nonzero. <!-- [^019f5-80ed3] -->
