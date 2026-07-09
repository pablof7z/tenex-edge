---
title: Tenex-Edge Home Directory
slug: tenex-edge-home-directory
topic: tenex-edge
summary: The `edge_home()` function returns tenex-edge's data root, including `state.db`, agents, and logs, and is overridable via `TENEX_EDGE_HOME`.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-07-03
verified: 2026-07-03
compiled-from: conversation
sources:
  - session:b07a57a3-67a1-4c44-a8fc-58a1bb97860a
  - session:019f12f9-8a0b-7012-ad2f-f4d0cb035d2b
---

# Tenex-Edge Home Directory

## Home Directory & Environment

The `edge_home()` function returns tenex-edge's data root — including `config.json`, `state.db`, agents, and logs — and is overridable via the `TENEX_EDGE_HOME` environment variable. When set, `TENEX_EDGE_HOME` overrides the default `~/.tenex-edge` data directory for all tenex-edge state. `config_path()` uses `edge_home()` to resolve the default config file location, so `TENEX_EDGE_HOME` is respected for config loading; `TENEX_CONFIG` can override only the config file. The separate `tenex_dir()` abstraction for shared TENEX platform config was removed; `providers.json`, `llms.json`, relay logs, identities, and daemon state all live under `edge_home()`. <!-- [^b07a5-1f7a8] -->

Processes spawned via `tenex-edge launch` inherit `TENEX_EDGE_HOME`, `TENEX_CONFIG`, and `TENEX_EDGE_BIN` from the parent environment, including agent harness hook calls. <!-- [^b07a5-ee2c8] -->

## Runtime Diagnostics

`tenex-edge` no longer prints the configured home directory and relay URL on every invocation. Use `tenex-edge debug doctor` when you need to inspect the active edge home, config path, and relay configuration.

## Related Skills

The `tenex-edge-store-read-model` skill teaches SQLite/read-model ownership: `relay_*` rebuildable caches vs local state, single daemon writer, store-as-reader-contract, and the future `StoreReader`/`StoreWriter` direction. <!-- [^019f1-cb88c] -->
