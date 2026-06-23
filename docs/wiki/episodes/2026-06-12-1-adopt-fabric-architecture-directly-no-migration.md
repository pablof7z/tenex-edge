---
type: episode-card
date: 2026-06-12
session: 0bc06206-1f30-4e35-8373-f31d0f5c1dcc
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/0bc06206-1f30-4e35-8373-f31d0f5c1dcc.jsonl
salience: architecture
status: active
subjects:
  - fabric-provider
  - canonical-store
  - strangler-pattern-migration
supersedes:
  - 2026-06-12-1-codec-seam-replaced-by-fabric-provider
related_claims: []
source_lines:
  - 420-438
  - 470-484
  - 486-487
captured_at: 2026-06-12T19:56:26Z
---

# Episode: Adopt fabric-architecture directly — no migration or backward compatibility

## Prior State

The fabric-architecture design doc (§6) prescribed a strangler-pattern migration: dual-write to legacy tables during cutover, then gradual removal of old access paths. The working assumption was that backward compatibility and a phased migration were required to land the refactor.

## Trigger

User directive at line 486: 'there's no migration needed… no backwards compatibility, just adopt it — just make sure it actually works by actually using it e2e'

## Decision

Drop the migration path entirely. The canonical store becomes the sole source of truth immediately; legacy dual-written tables are temporary scaffolding to be removed, not maintained for compatibility. Master's divergent features (NIP-10 threading, secret scrubbing, inbox envelopes, independent propose verb) must be reconciled into the new architecture rather than the new architecture accommodating old paths.

## Consequences

- Legacy tables (inbox, peer_sessions, agent_status, project_meta) are slated for deletion, not gradual phase-out
- The dual-write currently in state.rs is dead-end scaffolding — no migration consumers will ever read from it
- Master's 11 post-branch commits must be rebased/integrated onto the canonical-store architecture, not merged alongside
- The independent propose implementation on master (b201f4e) is superseded by the fabric branch's propose verb (08b35aa, b480295)
- E2e validation is the adoption gate, not migration completeness

## Open Tail

- Master divergence (~29 conflict hunks) must be resolved by threading recent features through canonical-store paths
- MLS identity binding, multi-fabric concurrency, and schema versioning remain open design questions (documented but not blocked)
- state.rs:1335 TODO naming the legacy-table removal phase needs a concrete plan now that migration is off the table

## Evidence

- transcript lines 420-438
- transcript lines 470-484
- transcript lines 486-487

