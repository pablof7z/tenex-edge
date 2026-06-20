---
title: tenex-edge Existing Nostr Fabric
slug: tenex-edge-existing-nostr-fabric
topic: tenex-edge
summary: A working Nostr agent fabric already exists on this machine, as demonstrated by the podcast-player app (Pod0)
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-07
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
---

# tenex-edge Existing Nostr Fabric

## Existing Nostr Fabric

A working Nostr agent fabric already exists on this machine, as demonstrated by the podcast-player app (Pod0). Pod0 uses relay.tenex.chat, TENEX-compatible event vocabulary, project coordinates, NIP-42 AUTH, and the NMP kernel for relay/signing abstraction — proving the network is real, not aspirational. The LLM agent loop in the podcast player is entirely decoupled from the Nostr layer: the Rust code provides transport, and Swift drives higher-level routing for autonomous responses. This decoupled architecture is a reusable pattern for tenex-edge. <!-- [^8a3eb-14] -->
