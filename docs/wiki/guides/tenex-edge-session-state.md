---
title: Tenex-Edge Session State
slug: tenex-edge-session-state
topic: tenex-edge
summary: The core architectural defect is that session state has no single owner; one logical fact (session S is about TITLE, doing ACTIVITY, busy B) physically lives in
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-16
updated: 2026-06-16
verified: 2026-06-16
compiled-from: conversation
sources:
  - session:1b868736-ed6b-4f88-84d9-26bb320accfd
---

# Tenex-Edge Session State

## Problem Statement

The core architectural defect is that session state has no single owner; one logical fact (session S is about TITLE, doing ACTIVITY, busy B) physically lives in ≥4 stores (task-local `cur_title`/`cur_activity`, `session_status`, legacy `agent_status`, and the kind:30315 tag), written from ~7 scattered sites with no single `apply(transition)` chokepoint. Additionally, session identity is borrowed, not minted — `session_id` is simultaneously the sqlite PK, the relay `d`-tag, the routing target, and the harness resume token, and its origin is unstable (opencode mints a fresh id every start). `DaemonState.last_status` is keyed by `(pubkey, project)`, which is structurally wrong for multi-session.

<!-- citations: [^1b868-13] [^1b868-23] [^1b868-41] -->
## Stable Session Identity

The session identity is a daemon-minted stable `session_key` derived from `agent+project+host+watch_pid`; harness-native IDs become aliases in a `session_aliases` table, making competing orphaned events structurally impossible.

<!-- citations: [^1b868-14] [^1b868-24] [^1b868-36] -->
## Single Source of Truth

A single `session_state` row (keyed by `session_key`) serves as the single source of truth for title, title_source, activity, phase, turn_started_at, last_distill_at, and last_seen, mutated only through explicit transition methods (`start_turn`, `seed_title`, `apply_distill`, `heartbeat`, `end`, `supersede`). A versioned `status_outbox` table with `(session_id, state_version)` and a publish drainer that retries and records the native event id makes a duplicate runtime or stale distill result structurally unable to flip the title even without the spawn guard. Legacy `agent_status` and `session_status` tables are deleted outright with no backwards compatibility.

<!-- citations: [^1b868-15] [^1b868-25] [^1b868-42] -->
## Liveness and Title Separation

Liveness (`last_seen`) and durable title are separated in the store, with liveness determined by a freshness window and title being durable. The kind:30315 event includes a NIP-40 expiration tag with a TTL of 90 seconds (matching `status_ttl`), re-armed every 30-second heartbeat, so that a stopped session's event expires off the relay automatically. No tombstone or terminal lifecycle events are used for session liveness; expiration alone signals death. (Previously: considering active tombstone publishing or relying on identity stability with a freshness window.)

<!-- citations: [^1b868-16] [^1b868-26] [^1b868-35] -->
## Deterministic Status Projection

One deterministic `derive_status(state, now) -> DerivedStatus` projection is shared by the publisher, `who.rs` (both local and peer branches), and the turn-context delta, eliminating the local-vs-peer fork.

<!-- citations: [^1b868-17] [^1b868-27] -->
## Local and Peer State Separation

Local state and peer state are structurally separated into `session_state` (keyed by `session_key`) and `peer_state` (keyed by pubkey/project/native), so the materializer physically cannot write local state and the `is_self` guard becomes unnecessary.

<!-- citations: [^1b868-18] [^1b868-28] -->
## Stateless Session Driver

The runtime task becomes a stateless `SessionDriver` whose `on_tick(now, store) -> Vec<Effect>` is a pure, table-testable transition over the persisted row, losing the local `cur_title`/`cur_activity` variables. The recommended approach combines Opus's pure `on_tick → Vec<Effect>` pattern for maximal table-testability with Codex's `status_outbox` for the publish path, as the two compose.

<!-- citations: [^1b868-19] [^1b868-29] -->
## Incremental Migration Plan

Migration proceeds incrementally on a single branch `session-state-rearchitecture` as a single PR with no backwards compatibility: Phase 0 extends the existing FREEZE test pattern to state invariants as failing-first oracles (one `d` per logical session across id rotation, title stable across restart, stale distill ignored, expired status not live); Phase 1 adds `session_key` + `session_aliases` and routes internally on the key; Phase 2 moves the `d`-tag to `session_key`; Phase 3 introduces `session_state` + `commit_session_state`; Phase 4 extracts `derive_status`; Phase 5 splits `peer_state`; Phase 6 resolves the canonical/legacy duality.

<!-- citations: [^1b868-20] [^1b868-30] [^1b868-43] -->
## Properties to Preserve

The good architectural properties to preserve are: single-writer daemon with no-await-holding-lock, the provider/codec seam with `Kind1Nip29Provider` as single publish chokepoint, the `DomainEvent` enum + pure `Kind1Codec` with roundtrip tests (`domain.rs` names zero Nostr concepts), mention routing + idempotency with inbox PK, `publish_checked`/`is_retrievable` for relay acceptance honesty, and the atomic `spawn_session` reserve.

<!-- citations: [^1b868-31] [^1b868-44] -->
## Current State Chain

The current session title is held in `cur_title: Option<String>`, a local variable in the per-session engine loop in `runtime.rs:149`. On every publish, the in-memory `cur_title` is written to the `text` column of the `session_status` table in SQLite, keyed by `(pubkey, project, session_id)`, and this persisted value survives a daemon restart. On the wire, the session title is placed in the `["title", title]` tag of the kind:30315 event, keyed replaceable by `d = "<project>:<session_id>"`. The full state chain is: `cur_title` (runtime memory) → `session_status.text` (sqlite) → `["title", …]` tag (relay event). <!-- [^1b868-32] -->

## d-Tag and Relay Keying

The `d`-tag for kind:30315 uses `session_key` (`<project>:<session_key>`) instead of harness-native `session_id`. <!-- [^1b868-37] -->
