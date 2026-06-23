---
title: OpenCode SQLite Schema Recovery
slug: opencode-sqlite-schema-recovery
topic: opencode-integration
summary: "When the opencode SQLite database has an older schema missing a `name` column, the process fails at startup with `no such column: name`"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-08
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:72c1c649-6826-4219-a8d4-b507abc78310
---

# OpenCode SQLite Schema Recovery

## Symptom: Stuck Startup with `no such column: name`

When the opencode SQLite database has an older schema missing a `name` column, the process fails at startup with `no such column: name`. It then falls into an idle event loop with zero open connections, making it appear stuck. <!-- [^72c1c-3] -->

## Fix: Back Up and Remove Database Files

Back up and remove all opencode database files — including WAL and shared-memory sidecar files — so a fresh schema is recreated on the next restart. This clears the broken older schema and resolves the stuck process. <!-- [^72c1c-4] -->
