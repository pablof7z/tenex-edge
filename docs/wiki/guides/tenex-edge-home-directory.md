---
title: Tenex-Edge Home Directory
slug: tenex-edge-home-directory
topic: tenex-edge
summary: The `edge_home()` function returns tenex-edge's data root â including `state.db`, agents, and logs â and is overridable via the `TENEX_EDGE_HOME` environmen
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-06-29
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:b07a57a3-67a1-4c44-a8fc-58a1bb97860a
  - session:019f12f9-8a0b-7012-ad2f-f4d0cb035d2b
---

# Tenex-Edge Home Directory

## Home Directory & Environment

The `edge_home()` function returns tenex-edge's data root — including `state.db`, agents, and logs — and is overridable via the `TENEX_EDGE_HOME` environment variable. When set, `TENEX_EDGE_HOME` overrides the default `~/.tenex-edge` data directory for all tenex-edge state: config, keys, state.db, and logs. `config_path()` uses `edge_home()` to resolve the config file location, so `TENEX_EDGE_HOME` is respected for config loading everywhere rather than hardcoding `~/.tenex-edge`. The separate `tenex_dir()` abstraction for the shared TENEX platform config (LLM configs, providers) overridable via `TENEX_DIR` has been removed; everything now reads from `edge_home()`. <!-- [^b07a5-1f7a8] -->

Processes spawned via `tenex-edge launch` inherit `TENEX_EDGE_HOME`, `TENEX_CONFIG`, and `TENEX_EDGE_BIN` from the parent environment, including agent harness hook calls. <!-- [^b07a5-ee2c8] -->


README and older architecture docs still mention `~/.tenex/edge`, while newer code and wiki guidance says `edge_home()`/`TENEX_EDGE_HOME` now own all tenex-edge state; treat the legacy path references as stale. <!-- [^019f1-c1479] -->
## Startup Diagnostics

On every `tenex-edge` invocation, before command dispatch, the configured home directory and relay URL are printed to stderr so stdout-based consumers like statusline are not affected. The line is formatted as `[tenex-edge] home=<home_path> relays=<relay_url>`. <!-- [^b07a5-df73d] -->

## Related Skills

The `tenex-edge-store-read-model` skill teaches SQLite/read-model ownership: `relay_*` rebuildable caches vs local state, single daemon writer, store-as-reader-contract, and the future `StoreReader`/`StoreWriter` direction. <!-- [^019f1-cb88c] -->
