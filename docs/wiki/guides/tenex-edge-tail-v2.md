---
title: tenex-edge Tail v2 Stream
slug: tenex-edge-tail-v2
topic: tenex-edge
summary: Tail v2 was implemented as a structured TailEvent stream with 10 variants, join/leave derivation from heartbeat suppression, 4 tiers, backfill, --json output, a
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
  - session:0bc06206-1f30-4e35-8373-f31d0f5c1dcc
---

# tenex-edge Tail v2 Stream

## Tail v2 Implementation

Tail v2 was implemented as a structured TailEvent stream with 10 variants, join/leave derivation from heartbeat suppression, 4 tiers, backfill, --json output, and 14 new unit tests. Tail emission is first-sight-gated so materialization always runs but only the tail broadcast is deduped, preserving load-bearing subscription replay. Self-authored events never derive a tail line, deterministically fixing the outbound/echo double-count race.

<!-- citations: [^ab999-22] [^0bc06-10] -->
