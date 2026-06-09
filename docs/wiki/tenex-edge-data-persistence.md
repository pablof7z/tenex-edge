---
title: Tenex-Edge Data Persistence
slug: tenex-edge-data-persistence
topic: tenex-edge
summary: Local state is stored in SQLite
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-09
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:72c1c649-6826-4219-a8d4-b507abc78310
  - session:ccdf5ab7-5155-4b5f-8be9-866a2608a8ee
  - session:162f9965-82ca-420b-aa24-99faa15cb59a
---

# Tenex-Edge Data Persistence

## Data Persistence

Local state is persisted in SQLite at ~/.tenex/edge/state.db. The channel server acts as a thin stream-consumer that never independently writes state.db, avoiding multi-writer corruption. A spike is needed early to determine how NMP handles embedding in N concurrent per-session processes, specifically whether LMDB requires separate paths per process or supports a shared mode. OpenCode stalls on startup when its SQLite database schema is missing a 'name' column that the current version expects. To recover from a broken database schema, the database file must be renamed with a .bak suffix so that the process recreates it fresh on startup.

<!-- citations: [^f3a73-29] [^f3a73-49] [^72c1c-1] [^ccdf5-1] [^162f9-7] -->
