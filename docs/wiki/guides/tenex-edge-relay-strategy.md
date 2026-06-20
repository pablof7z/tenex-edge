---
title: tenex-edge Relay Strategy
slug: tenex-edge-relay-strategy
topic: tenex-edge
summary: The recommended relay strategy is a personal relay per operator (single propagation domain, small settle window), supplemented by shared collaboration relays fo
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-16
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
  - session:f9bdcf4c-c972-46ff-91b8-9e30785d3331
  - session:d683a556-03b8-4827-b84d-5395cd3610af
  - session:rollout-2026-06-16T14-11-38-019ed021-38a8-7472-bc5d-dc019a072086
---

# tenex-edge Relay Strategy

## Relay Strategy

The recommended relay strategy is a personal relay per operator (single propagation domain, small settle window), supplemented by shared collaboration relays for cross-person goals/resources and NIP-65 for discovery. The default relay is `wss://nip29.f7z.io` (Previously: `relay.tenex.chat`.) Tests reference the default relay via the `DEFAULT_RELAY` constant rather than hard-coding the relay string. The live user config at `~/.tenex/config.json` explicitly specifies the relay list (`"relays": ["wss://nip29.f7z.io"]`) rather than relying on the compiled-in default. Users can add fallback relays via the relays field in `~/.tenex/config.json` (Config.relays is a Vec<String> and Transport::connect adds them all, so no code change is needed for multiple-relay support). Note that `wss://nip29.f7z.io` rejects writes for `kind:1` events, blocking them with the error 'blocked: kind 1 is not allowed; timeout'.

<!-- citations: [^8a3eb-27] [^f9bdc-3] [^d683a-5] [^rollo-78] -->
