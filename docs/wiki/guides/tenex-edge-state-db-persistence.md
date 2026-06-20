---
title: tenex-edge state.db Persistence Architecture
slug: tenex-edge-state-db-persistence
topic: data-persistence
summary: SQLite multi-writer corruption is a risk because approximately 16 per-session engines plus CLI invocations all share one state.db
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-16
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:05b89548-666c-4e24-a2f5-8a1e92f0bf04
  - session:633f8f7f-37f8-409c-90a9-ef64b0dc3216
  - session:rollout-2026-06-16T17-43-45-019ed0e3-68e5-7091-899d-6a4e0fcb5716
  - session:ses_13a5173feffeXR4Fi4UffHR88M
  - session:ses_13a5107b0ffeS3nHRuWFcAx21V
---

# tenex-edge state.db Persistence Architecture

## Multi-Writer Persistence Flaw

SQLite multi-writer corruption is a risk because approximately 16 per-session engines plus CLI invocations all share one state.db. WAL mode plus `busy_timeout(5s)` plus `synchronous=NORMAL` is a stopgap, not a fix, and real corruption has occurred. N per-session processes writing one shared state.db file is a confirmed failure mode that will recur, requiring an architectural fix. The architectural fix options for the multi-writer persistence flaw are: a single-writer daemon owning state.db with sessions communicating over IPC, per-session DB files with no sharing, or hardening the shared file with identical binary versions, busy_timeout, and no truncate/VACUUM. The pre-daemon `run_session` function in `runtime.rs` and CLI invocations each open their own `Store::open()` connection to `state.db`, bypassing the daemon's `Arc<Mutex<Store>>` centralized write path. Everything in state.db (peer presence, agent_status, seen_mentions, turn_state, pending agents) is ephemeral and reconstructible from the live relay, so a fresh empty DB repopulates within a heartbeat or two. Legacy tables (`inbox`, `peer_sessions`, `agent_status`, `project_meta`) are slated for deletion but are deeply embedded in the codebase — `inbox` alone is referenced in 100+ locations as the primary delivery mechanism for mentions. Comments in `state.rs` label these canonical tables as "deliberately-retained canonical home," contradicting the architectural decision that they are slated for deletion. The dual-write code in `state.rs` and `provider.rs` is dead-end scaffolding — canonical table writes in `provider.send()`, `rpc_propose()`, and the materializer write rows that nobody reads, because the migration path was dropped and read paths still use legacy tables. The `.ok()` pattern is used 50+ times on store writes (e.g., `upsert_session`, `touch_session`, `mark_turn_end`, `set_agent_status`), silently discarding database write failures; canonical writes are best-effort (`.ok()`, never failing the legacy path), meaning canonical read models can silently diverge from reality. A session_errors table (one row per session, upserted on each failure) in state.rs provides record_session_error and get_recent_session_error methods. Existing populated state.db sessions are not migrated into session_state, so upgrade/restart does not preserve live sessions. Transition methods execute multiple standalone statements rather than operating as actual SQLite transactions.

<!-- citations: [^05b89-3] [^05b89-4] [^05b89-5] [^633f8-4] [^rollo-88] [^rollo-89] [^ses_1-6] [^ses_1-15] -->
## Recovery Convention

A .bak suffix on state.db indicates a previously-broken DB renamed per the documented recovery convention (so the process recreates it fresh), not a deliberate clean backup. <!-- [^05b89-6] -->

## Plugin Bootstrap Implications

The persistence architecture decision (single-writer daemon vs per-session DB vs hardened shared file) shapes what the Claude Code plugin's bootstrap hook installs or connects to. <!-- [^05b89-7] -->
