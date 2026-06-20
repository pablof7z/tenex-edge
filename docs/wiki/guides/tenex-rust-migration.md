---
title: TENEX Rust Migration
slug: tenex-rust-migration
topic: architecture
summary: TENEX has been fully migrated from TypeScript/Bun to 100% Rust, with the TypeScript runtime removed and a read-only reference preserved at a specific path
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-14
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:162f9965-82ca-420b-aa24-99faa15cb59a
  - session:74fce09f-02b4-496f-a5e1-52d19ef9fbcd
  - session:rollout-2026-06-12T11-18-49-019ebae9-8fa7-73f1-844d-bea23bfb0193
  - session:rollout-2026-06-14T13-19-49-019ec5a5-1119-76f0-a7e3-36bc985a31bd
---

# TENEX Rust Migration

## TENEX Rust Migration

TENEX has been fully migrated from TypeScript/Bun to 100% Rust, with the TypeScript runtime removed and a read-only reference preserved at a specific path. The reqwest dependency uses the rustls feature name (not rustls-tls) to compile against the locked 0.13.4 version. The rustls CryptoProvider must be explicitly installed at process startup (ring provider) because rig-core's reqwest pulls aws-lc-rs alongside ring, and rustls 0.23 panics when both are compiled in. Uncompiled daemon, state, and runtime split files are deleted rather than wired into the build, keeping the working monoliths as the source of truth. The node_modules/ directory is excluded from git via .gitignore and can be restored using bun install from the committed bun.lock. The AdSegmentDetectorTests.swift file was deleted because AIChapterCompiler no longer exists, as chapter and ad generation has moved to the Rust kernel.

<!-- citations: [^8a3eb-35] [^f3a73-26] [^162f9-9] [^74fce-17] [^rollo-44] [^rollo-52] -->
