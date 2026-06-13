---
title: Tenex-Edge Relay Configuration
slug: tenex-edge-relay-configuration
topic: tenex-edge
summary: Presence, activity, status, and mention events all use the NIP-29 h tag with the project slug as a namespace filter, replacing the previous T tag
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-09
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:f9bdcf4c-c972-46ff-91b8-9e30785d3331
  - session:98f9939c-f42b-43dd-baba-d9a176d4b2d7
  - session:36cc4546-228e-4d07-a1a8-9d0cd7cd5a6c
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:rollout-2026-06-09T10-55-30-019eab61-23ae-7163-8d06-9a3965847e4f
---

# Tenex-Edge Relay Configuration

## Default Relay

Presence, activity, status, and mention events all use the NIP-29 h tag with the project slug as a namespace filter, replacing the previous T tag. Project-scoped live traffic is subscribed via #h filters and is not restricted to a local author allowlist, reflecting open NIP-29 group behavior. Missing h tags on events result in rejection rather than decoding with an empty project. Status uses kind:30315 with a d tag equal to the project slug (one replaceable event per project), distinct from presence which uses a per-session d tag. The default relay is wss://nip29.f7z.io (changed from relay.tenex.chat now that nip29 is in use); the compiled-in DEFAULT_RELAY in src/config.rs is wss://nip29.f7z.io. The relay is the remote bridge between phone and daemon; push delivery to a backgrounded phone (via APNs/FCM) is deferred as unsolved. The refactored tenex-edge binary and the tenex-off app must share the same relay to close the loop; a relay mismatch means the app can't see proposals and comments can't reach agents. Transport::connect forces NIP-42 AUTH completion with a fetch_events warm-up before any subscribe, because the relay requires AUTH for reads and subscriptions opened pre-auth are closed. The live config at ~/.tenex/config.json explicitly declares the relay as wss://nip29.f7z.io (the relays array including wss://nip29.f7z.io) rather than relying on the compiled-in default. A signed event published over a differently-authenticated relay connection lands under the signing key's authorship (publish_signed method). The project list and project edit target nip29.f7z.io, a proper NIP-29 relay that tracks kind:39000 group metadata and expects kind:9002 for updates. The doctor probe publishes and reads back using #h to test the same group tag path as normal traffic. Publishing a proposal to a closed, auth-gated NIP-29 relay such as nip29.f7z.io fails for unauthorized or unknown agent keys, whereas open relays such as relay.primal.net accept them but may exhibit ingestion lag. The live external nip29_probe test (wss://nip29.f7z.io) is not run as a regression oracle because it probes relay rules, not this codebase's behavior.

<!-- citations: [^ab999-55] [^f9bdc-2] [^98f99-17] [^98f99-22] [^36cc4-5] [^ab999-40] [^ab999-62] [^f9bdc-5] [^98f99-32] [^rollo-7] -->
