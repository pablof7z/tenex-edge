---
title: tenex-edge Recipient Resolution
slug: tenex-edge-recipient-resolution
topic: tenex-edge
summary: `resolve_recipient` accepts `agent@host`, raw pubkeys, session ids/aliases/prefixes/codenames, and bare local agent slugs for daemon chat delivery.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-16
updated: 2026-06-23
verified: 2026-06-23
compiled-from: conversation
sources:
  - session:ses_13081afccffeSOadIDwUtF3Sfz
  - session:ses_1307cfa82ffezNqP0fk6nYNJvs
---

# tenex-edge Recipient Resolution

## Resolver Order

The daemon's `resolve_recipient` path is used for chat delivery targets, including inline `@codename` mentions. It parses target strings through `idref` in this order:

- `agent@host` resolves through local/peer profile state for that agent on that host.
- A 64-hex key or `npub` resolves directly to the raw pubkey.
- A bare token first tries exact canonical session id or harness alias, then a session-id prefix, then a displayed session codename.
- A remaining bare token is treated as an agent slug on the local host.

Session targets p-tag the durable agent pubkey and also carry `target_session` for local delivery. Bare agent targets route to the local host's durable agent key. NIP-29 group membership is not the source for recipient resolution; it has pubkeys but no agent slugs or session codenames.
