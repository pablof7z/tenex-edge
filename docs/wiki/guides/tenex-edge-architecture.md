---
title: Tenex-Edge Architecture
slug: tenex-edge-architecture
topic: tenex-edge
summary: Tenex-edge is a Rust project (38.5k LOC in src/) providing durable Nostr-keypair identity, presence, and cross-agent messaging for AI coding-agent sessions via
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-07-03
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:019f12f9-8a0b-7012-ad2f-f4d0cb035d2b
  - session:75f62bb9-f564-4633-8741-997dfea1d0e7
---

# Tenex-Edge Architecture

## Core Implementation Contract

Tenex-edge is a Rust project (38.5k LOC in src/) providing durable Nostr-keypair identity, presence, and cross-agent messaging for AI coding-agent sessions via a per-machine daemon. The fabric is the thin, always-on Nostr-based layer floating above every terminal where agents exist as persistent beings with presence, activity, and @mention capability while hosts remain disposable. The core implementation contract consists of pure domain types, a NIP-29 wire codec, a provider/materializer, a single-writer daemon, and a SQLite read model.

The hook is a host-neutral integration point that shells out to the tenex-edge binary on session start and on every user turn, piping JSON to the daemon; each host (Claude Code, Codex, opencode, Grok) is a thin adapter that knows nothing about tenex-edge's internals. <!-- [^75f62-76b60] -->

<!-- citations: [^019f1-a0205] [^75f62-469f0] [^75f62-c3c28] -->
