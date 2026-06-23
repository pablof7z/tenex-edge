---
title: Tenex-Edge Thread Storage
slug: tenex-edge-thread-storage
topic: tenex-edge
summary: The old threads read model and `tenex-edge threads` CLI are removed on current master; proposal publication now goes through `tenex-edge publish`.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-10
updated: 2026-06-23
verified: 2026-06-23
compiled-from: conversation
sources:
  - session:56f9fe89-5ff7-4e5b-b202-334cd7629d42
  - session:40a4d401-2520-4781-b747-b0ef19594bed
---

# Tenex-Edge Thread Storage

## Threads Plane Removed

The former threads read model and `tenex-edge threads` command are not present on current master. The thread tables/RPCs were removed, and proposal publication now uses `tenex-edge publish` to publish kind:30023 proposals directly.

Current agent communication uses project chat (`chat write` / `chat read`) and per-session room chat. Any future threaded conversation browser needs a new current design rather than depending on the old `threads`, `messages`, or `thread_meta` surfaces.
