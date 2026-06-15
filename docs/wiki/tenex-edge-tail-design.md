---
title: Tenex-Edge Tail Design
slug: tenex-edge-tail-design
topic: tenex-edge
summary: The tail v2 command provides a structured TailEvent stream with 9 variants (Msg, Sync, Turn, Join, Leave, Sess, Status, Proj, Profile), heartbeatâjoin/leave d
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-14
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:rollout-2026-06-14T13-17-10-019ec5a2-a38e-7403-906f-836d766d9291
---

# Tenex-Edge Tail Design

## Tail v2 Design

The tail v2 command provides a structured TailEvent stream with 9 variants (Msg, Sync, Turn, Join, Leave, Sess, Status, Proj, Profile), heartbeat→join/leave derivation, dedup for Profile and Status, read-model backfill, and 17 CLI flags including --project, --since, --only, --exclude, --json, --compact. The Acl tail event category and its CLI renderer branch are removed from the tail command.

<!-- citations: [^ab999-14] [^ab999-92] [^rollo-31] -->
