---
title: Tenex-Edge Transport Codec
slug: tenex-edge-transport-codec
topic: tenex-edge
summary: Envelope encoding and decoding is modularized as a codec set providing per-event encode, decode, and subscribe operations
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-08
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
---

# Tenex-Edge Transport Codec

## Codec Set Architecture

Envelope encoding and decoding is modularized as a codec set providing per-event encode, decode, and subscribe operations, decoupling envelope shapes from business logic so that alternative transports (such as another NIP-29 relay policy or Marmot/MLS) can be added as additional codec adapters without modifying domain logic. The codec set is granular per event type rather than a monolith. Heartbeat activity is included in the codec set because its shape varies across different codecs. The initial shape adapter uses kind:1 notes and kind:30315 heartbeats/statuses, all project-scoped with the NIP-29 `h` tag.

Domain code must never name a specific Nostr kind, tag, or wire-protocol concept, keeping the domain layer transport-agnostic.

NIP-42 AUTH must be built into the transport layer from day one, since relays almost certainly require it for publishes and silently reject publishes without it. Additionally, transport must force NIP-42 AUTH completion (a warm-up fetch) before any subscribe, because relay.tenex.chat requires auth for reads and closes subscriptions opened before auth completes.

Transport is built on the lean `nostr`/`nostr-sdk` stack behind a Transport trait, rather than embedding the full NMP kernel (which is unsuitable for headless CLI daemons), with NMP documented as the intended future swap-in behind the codec seam.

The runtime handles nostr-sdk's dedup by also processing raw Message event variants, not just the deduped Event notification, otherwise pending-discovery silently fails.

<!-- citations: [^f3a73-36] [^f3a73-37] [^f3a73-38] [^f3a73-46] [^f3a73-58] [^f3a73-79] [^f3a73-85] [^f3a73-97] -->
