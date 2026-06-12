---
title: Tenex-Edge Tail Design
slug: tenex-edge-tail-design
topic: tenex-edge
summary: The tail v2 command provides a structured TailEvent stream with 10 variants (Msg, Sync, Turn, Join, Leave, Acl, Sess, Status, Proj, Profile), heartbeatâjoin/l
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-12
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:ab9998c4-6e65-410e-b298-122a2072171c
---

# Tenex-Edge Tail Design

## Tail v2 Design

The tail v2 command provides a structured TailEvent stream with 10 variants (Msg, Sync, Turn, Join, Leave, Acl, Sess, Status, Proj, Profile), heartbeat→join/leave derivation, dedup for Profile and Status, read-model backfill, and 17 CLI flags including --project, --since, --only, --exclude, --json, --compact.

<!-- citations: [^ab999-14] [^ab999-92] -->
