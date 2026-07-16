---
title: Mosaico Home Directory
slug: mosaico-home-directory
topic: mosaico
summary: The `mosaico_home()` function returns mosaico's data root, including `state.db`, agents, and logs, and is overridable via `MOSAICO_HOME`.
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

# Mosaico Home Directory

## Home Directory & Environment

The `mosaico_home()` function returns mosaico's data root — including `config.json`, `state.db`, agents, and logs — and is overridable via the `MOSAICO_HOME` environment variable. When set, `MOSAICO_HOME` overrides the default `~/.mosaico` data directory for all mosaico state. `config_path()` uses `mosaico_home()` to resolve the default config file location, so `MOSAICO_HOME` is respected for config loading; `MOSAICO_CONFIG` can override only the config file. <!-- [^b07a5-1f7a8] -->

Processes spawned via `mosaico launch` inherit `MOSAICO_HOME`, `MOSAICO_CONFIG`, and `MOSAICO_BIN` from the parent environment, including agent harness hook calls. <!-- [^b07a5-ee2c8] -->

## Runtime Diagnostics

Use `mosaico debug doctor` when you need to inspect the active Mosaico home, config path, and relay configuration.

## Related Skills

The `mosaico-store-read-model` skill teaches SQLite/read-model ownership: `relay_*` rebuildable caches vs local state, single daemon writer, store-as-reader-contract, and the future `StoreReader`/`StoreWriter` direction. <!-- [^019f1-cb88c] -->
