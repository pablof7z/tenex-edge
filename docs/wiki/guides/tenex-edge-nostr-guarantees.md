---
title: tenex-edge Nostr Guarantees
slug: tenex-edge-nostr-guarantees
topic: tenex-edge
summary: Nostr is an AP (available, partition-tolerant) system; relays are an eventually-consistent gossip bus with no compare-and-swap, no broadcast guarantee, and no e
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-08
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
---

# tenex-edge Nostr Guarantees

## System Model & Guarantees

Nostr is an AP (available, partition-tolerant) system; relays are an eventually-consistent gossip bus with no compare-and-swap, no broadcast guarantee, and no enforced TTL, so tenex-edge provides only advisory coordination computed client-side, never mutual exclusion. relay.tenex.chat requires NIP-42 AUTH for reads—subscriptions opened before auth completes are silently closed—so Transport::connect forces an AUTH warm-up before any subscribe. The engine handles both nostr-sdk's deduplicated Event notification and the raw Message::Event variant, because the auth warmup marks events as seen, which would otherwise cause dedup suppression.

<!-- citations: [^8a3eb-21] [^f3a73-23] -->
