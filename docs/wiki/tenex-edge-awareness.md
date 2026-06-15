---
title: Tenex-Edge Awareness
slug: tenex-edge-awareness
topic: tenex-edge
summary: Tenex-edge provides awareness of shared active work, goals, and access to resources
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-15
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:162f9965-82ca-420b-aa24-99faa15cb59a
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
  - session:rollout-2026-06-09T10-55-30-019eab61-23ae-7163-8d06-9a3965847e4f
  - session:a0037729-ad51-460a-880d-0a9699f6ee41
---

# Tenex-Edge Awareness

## Transport Architecture

Tenex-edge provides awareness of shared active work, goals, and access to resources. The board's state model lives behind a transport interface so that lifting the same state onto the network fabric is a transport swap rather than a rewrite. Envelope shapes are decoupled from business logic via a modularized encoder/decoder (codec set), allowing alternative transports (e.g., NIP-29, Marmot/MLS) to be added as separate shape adapters without modifying domain logic. Rung 0 transport uses local SQLite; Rung 1 swaps to the proven Nostr kernel.

<!-- citations: [^f3a73-4] [^f3a73-8] [^f3a73-17] [^8a3eb-25] -->
## Collision Logging

Q1 collision logging lives entirely inside tenex-edge's awareness model as the substrate observing activity reported across its boundary, independent of any specific host. The passive collision logger records (agent, path, timestamp) with no coordination logic to gather data for Rung 2 decisions. <!-- [^f3a73-9] -->

## Agent Activity Events

Agent activity is published as Nostr kind 1 events with a NIP-29 `h` tag whose value is the project slug, plus human-readable intent-level content. Activity distillation is auto-distilled (not agent-manual), using an LLM step to produce intent-level awareness lines. Agents maintain a running NIP-38 status per project, `h`-tagging the project slug and retaining a `d` value for replacement identity, with the status empty when idle. NIP-38 status events include a NIP-40 expiration tag so that stale status clears even if the process dies unexpectedly.

The cwd field is added to the status event so that the `who` renderer can display working directories. The project-relative form of cwd is put on the wire (not absolute $HOME paths) to mitigate leaking filesystem paths in world-readable public events on relay.tenex.chat. <!-- [^162f9-10] -->

<!-- citations: [^f3a73-18] [^f3a73-23] -->

## Presence Events

Presence is published every 30 seconds as an expiring `kind:30315` heartbeat with `h` set to the project slug, `d` set to `tenex-edge-presence:<session-id>`, `session-id` carrying the host session ID, and `expiration` bounding liveness. Slug is not carried on the wire; it is resolved from the signer's kind:0 Profile by pubkey. <!-- [^f3a73-24] -->

## Tail Client

A `tenex-edge tail -f <optional-project-slug>` command provides a colorized streaming client of all awareness activity. <!-- [^f3a73-25] -->

## Legacy Migration

Once tenex-edge's awareness board is live, pc's legacy awareness module will be removed. PC's awareness hooks and session-start are deleted, and pc keeps only inject + capture, consuming awareness deltas from tenex-edge instead.

<!-- citations: [^f3a73-26] [^f3a73-87] [^f3a73-111] -->
## `who` Command

The `who` command displays agents as: agent@hostname [session $id] [$relativePwd] followed by their current status, where relativePwd is relative to the project root (showing `.` for the root). Agents on the same machine show no host annotation; agents on a different host are annotated with (remote). The one-shot who output shows agent@project (not agent@host). The `who --live` command opens a full-screen refreshing terminal board over the local awareness snapshot. The `--live` board shows columns for AGENT@HOST, project, status, session, and seen age. The `--live` board exits cleanly on q, Esc, or Ctrl-C. The `--all` flag combined with `--live` keeps stale sessions visible. The `--refresh-ms` flag controls the refresh speed of the live board.

<!-- citations: [^162f9-11] [^rollo-5] -->

## PostToolUse Awareness Delta

The PostToolUse hook provides delta-gated, project-scoped awareness of sibling sessions' title and activity changes, emitting output only when something changed in the current project since the last check, and never about the user's own session. The delta is debounced with a 60-second floor: a check only runs if at least 60 seconds have elapsed since the last injection, though the first check of a turn always fires immediately. The delta query reuses `list_status_changes_since` and `list_new_peer_sessions`, scoped to the current project and excluding the current session ID. `list_status_changes_since` returns the activity field in addition to the title and session metadata. The delta includes the session's title, its activity line, and idle transitions (rendered as `· idle`); idle transitions bump `updated_at` so they are caught by `list_status_changes_since` and rendered as `<title> · idle`. The delta cursor (`turn_state.last_check_at`) resets to 0 at each turn start and advances to the current timestamp after each check that actually runs; the cursor write happens inside the daemon (single-writer via daemon-mediated RPC), avoiding multiwriter risk. The `turn_check_due()` function fails silent (returns `None`) when the session is not mid-turn, preventing PostToolUse from running out-of-turn. Direct inbox messages still surface immediately in PostToolUse output and are not subject to the 60-second debounce rate limit. The PostToolUse hook process timeout in settings.json is set to 10 seconds to match the per-tool-call cadence and fail fast if the daemon hangs. <!-- [^a0037-1] -->
