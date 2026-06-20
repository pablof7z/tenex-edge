---
type: episode-card
date: 2026-06-07
session: 8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666.jsonl
salience: root-cause
status: active
subjects:
  - strategic-posture
  - existing-fabric-reuse
  - proactive-context-ancestor
supersedes: []
related_claims: []
source_lines:
  - 700-724
  - 883-1082
  - 1096-1101
captured_at: 2026-06-17T23:35:21Z
---

# Episode: Not greenfield — tenex-edge is the on-ramp to an existing fabric

## Prior State

tenex-edge was assumed to be a greenfield product — a new network and coordination layer to be built from scratch.

## Trigger

Architecture agent discovered that proactive-context (pc, already wired into ~/.claude/settings.json) is a Rust+SQLite hook-driven sidecar whose awareness.rs is a single-device cross-agent standup board — the local-only ancestor of tenex-edge. Podcast-player recon confirmed a running Nostr agent fabric already on relay.tenex.chat with TENEX-compatible event vocabulary and project coordinates.

## Decision

tenex-edge is the 'missing membrane' — the on-ramp that lets a foreign-hosted agent become a first-class citizen of an already-operating fabric. Strategic posture changed from 'build a new product' to 'extend and generalize what already works.'

## Consequences

- proactive-context's hook-shim discipline (fire-and-forget, exit-0-always) is the proven integration model to reuse, not reinvent
- The architecture split (thin stateless shims → UDS → persistent daemon) is delta from pc, not a new design
- Phase 0 is local-only (zero Nostr) — prove presence+lock on one device using existing pc infrastructure before adding relay sync
- Podcast-player's NMP kernel pattern (app code never holds keys or opens sockets) is the model for transport/identity separation
- The TENEX event vocabulary and project-coordinate conventions are adopted as-is, not redesigned

## Open Tail

- Exact reuse boundaries from pc codebase still to be determined (copy? fork? depend?)
- Whether NMP kernel can be extracted as a shared crate or must be re-implemented

## Evidence

- transcript lines 700-724
- transcript lines 883-1082
- transcript lines 1096-1101

