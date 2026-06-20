---
title: tenex-edge Technical Debt
slug: tenex-edge-technical-debt
topic: tenex-edge
summary: Technical debt identification is scoped to identification only, with no changes to be made
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-14
updated: 2026-06-14
verified: 2026-06-14
compiled-from: conversation
sources:
  - session:ses_13a5173feffeXR4Fi4UffHR88M
  - session:ses_13a5107b0ffeS3nHRuWFcAx21V
---

# tenex-edge Technical Debt

## Scope

Technical debt identification is scoped to identification only, with no changes to be made. <!-- [^ses_1-7] -->

## God-Object Files

God-object files exist: state.rs (3453 lines, holding all CRUD operations, schema definitions, read-model accessors, dual-write scaffolding, in-memory query helpers, and 900+ lines of tests), cli.rs (2988 lines, mixing CLI argument parsing, subcommand dispatch, live terminal UI, turn context assembly, inbox display, session hooks, and inline who-snapshot tests), and daemon/server.rs (2791 lines, containing DaemonState, UDS listener, 30+ RPC handlers, peer tracking, relay event demux, and duplicate resolve_pubkey_hex).

<!-- citations: [^ses_1-8] [^ses_1-16] -->
## Dual-Write Dead Scaffolding

Dual-write dead scaffolding exists where provider.send() writes canonical rows (projects, threads, messages, etc.) that nobody reads, all read paths still use legacy tables, and the migration path was dropped making this code write-only overhead. <!-- [^ses_1-9] -->

## Panicky Error Handling

Panicky error handling exists with .unwrap() on signing (transport.rs:395, could panic on signing failure), .expect() on serialization (nostr_delivery.rs:129, could panic on serde_json serialization failure), and .expect() on identity load (server.rs:2467, could panic if files corrupt), plus .ok() silently swallows store write failures on 50+ call sites.

<!-- citations: [^ses_1-10] [^ses_1-20] -->
## Best-Effort Canonical Writes

Best-effort canonical writes use .ok() on canonical table inserts, causing silent divergence from reality. <!-- [^ses_1-11] -->

## Duplicated Logic

resolve_pubkey_hex is duplicated identically in src/daemon/server.rs line 1780 and src/daemon/server/admin.rs line 233. Additionally, cli.rs contains load_who_snapshot() and push_turn_fabric_block() which are called by the daemon server, creating a backwards dependency from daemon → cli — the CLI should be a thin client over the daemon, not a library the daemon imports.

<!-- citations: [^ses_1-12] [^ses_1-17] -->
## Mixed Error Patterns

Mixed error patterns exist using anyhow::Result, anyhow::bail!(), .ok() suppression, .unwrap() panics, and .expect() across production code, with no custom error types.

<!-- citations: [^ses_1-13] [^ses_1-19] -->
## Partial Submodule Extraction

daemon/server.rs still has 2692 lines with many unextracted RPC handlers, creating confusing duplication where submodule files are the live versions called from connection.rs dispatch but server.rs retains its own copies. <!-- [^ses_1-18] -->
