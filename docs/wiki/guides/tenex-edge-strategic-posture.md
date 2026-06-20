---
title: tenex-edge Strategic Posture
slug: tenex-edge-strategic-posture
topic: tenex-edge
summary: The Tenex Edge distribution mechanism is strictly an adapter that external systems depend on, never the reverseâthe word 'plugin' must not leak into tenex-edg
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-09
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
  - session:05b89548-666c-4e24-a2f5-8a1e92f0bf04
---

# tenex-edge Strategic Posture

## Strategic Posture

The Tenex Edge distribution mechanism is strictly an adapter that external systems depend on, never the reverse—the word 'plugin' must not leak into tenex-edge's vocabulary. The durable asset is the fabric/identity layer: the Nostr coordination spec, relay conventions, and key management. If a host absorbs the adapter, the fabric remains the place where cross-host and cross-person identity lives.

<!-- citations: [^8a3eb-29] [^05b89-8] -->
