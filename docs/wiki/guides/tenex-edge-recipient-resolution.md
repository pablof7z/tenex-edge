---
title: tenex-edge Recipient Resolution
slug: tenex-edge-recipient-resolution
topic: tenex-edge
summary: When the `--to` argument contains an `@`, `resolve_recipient` parses it as a slug and project qualifier, then calls `store.resolve_agent_pubkey(slug, Some(proj)
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-16
updated: 2026-06-16
verified: 2026-06-16
compiled-from: conversation
sources:
  - session:ses_13081afccffeSOadIDwUtF3Sfz
  - session:ses_1307cfa82ffezNqP0fk6nYNJvs
---

# tenex-edge Recipient Resolution

## Recipient Resolution Fallback

When the `--to` argument contains an `@`, `resolve_recipient` parses it as a slug and project qualifier, then calls `store.resolve_agent_pubkey(slug, Some(proj))`. The `resolve_agent_pubkey` method only queries Nostr relay-sourced tables (`peer_sessions` and `profiles`); it does not check the local agent keystore. When `resolve_agent_pubkey` returns `None`, `resolve_recipient` must fall back to the local agent keystore before producing an error. This fallback bridges the gap via a function like `identity::resolve_local_agent_pubkey(edge_home, slug)` that reads `<slug>.json` and returns its `public_key`. If resolution ultimately fails, the error is formatted as "can't resolve {slug}@{proj} (no presence/profile seen yet)".

The part after `@` in a recipient like `slug@host` is ambiguous — it can be a hostname (from local agents) or a project name (from relay data). Because local agent keystore files are keyed by slug only, the local keystore lookup must ignore or reinterpret the project parameter when the part after `@` is a host name rather than a project. (Previously: the local keystore fallback ignores the `@` suffix.)

NIP-29 group_members is not the source for `resolve_agent_pubkey`; the source is kind:0 profile events and kind:30315 status events. NIP-29 group_members has pubkeys but no slugs, making it an unused potential fallback for recipient resolution.

<!-- citations: [^ses_1-28] [^ses_1-29] [^ses_1-36] -->
