---
type: episode-card
date: 2026-06-07
session: 8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666.jsonl
salience: root-cause
status: active
subjects:
  - tenex-edge
  - proactive-context
  - podcast-player
  - strategic-positioning
supersedes: []
related_claims: []
source_lines:
  - 695-725
  - 800-826
  - 1096-1100
captured_at: 2026-06-12T19:48:00Z
---

# Episode: Not greenfield — existing fabric makes tenex-edge an on-ramp, not a new product

## Prior State

Tenex-edge was assumed to be a greenfield product — something to be built from scratch for a network that doesn't yet exist.

## Trigger

Architecture agent discovered that `proactive-context` (`pc` binary) is already wired into the user's Claude Code as a Rust+SQLite hook-driven sidecar with an awareness board (`awareness.rs`), and the podcast-player already runs on relay.tenex.chat with TENEX-compatible vocabulary and project coordinates.

## Decision

Tenex-edge is the missing membrane — the on-ramp that lets a foreign-hosted agent (running in someone else's tool) become a first-class citizen of an already-existing fabric. It builds on `proactive-context`'s hook discipline + awareness board pattern, and the podcast-player's NMP abstraction for transport/signing/relay routing.

## Consequences

- The hook-shim pattern (thin, stateless, fire-and-forget) from proactive-context is reused verbatim
- awareness.rs's standup-board model generalizes from local SQLite to Nostr-relayed fabric
- Phase 0 needs zero Nostr — prove presence+lock locally first, then make the relay the growth axis
- NMP's abstraction (app code never holds keys or opens sockets) is the architectural model for tenex-edge's daemon
- The podcast-player's feedback-handler pattern (NIP-72 project anchor + NIP-70 protection) is a reusable blueprint for gating coordination events

## Open Tail

*(none)*

## Evidence

- transcript lines 695-725
- transcript lines 800-826
- transcript lines 1096-1100

