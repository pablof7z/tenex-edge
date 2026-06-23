---
type: episode-card
date: 2026-06-09
session: d208c058-7b2b-4ff8-bb82-d63623d51097
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d208c058-7b2b-4ff8-bb82-d63623d51097.jsonl
salience: architecture
status: active
subjects:
  - fabric-architecture
  - codec-seam
  - read-model
  - provider-materializer
supersedes: []
related_claims: []
source_lines:
  []
captured_at: 2026-06-12T20:12:01Z
---

# Episode: CQRS read-model-is-contract replaces provider-through-reads

## Prior State

Reads (roster, project_meta, list_agents) flowed through the provider; the Codec trait was nostr-coupled (EventBuilder, Event, Filter are nostr_sdk types), so any 'new codec' could only ever be another nostr wire-shape.

## Trigger

User correction: 'it's still quite coupled to nostr? I would think the taxonomy would be abstract enough to support any other transport' — and the earlier correction that reads should not go through the provider.

## Decision

The read model is the contract. The provider is a write-side materializer. How data was hydrated is invisible to every reader. The Materializer (capability ②) now composes Wire-codec (③) + Delivery (④) rather than re-owning decode/subscribe. Provider bundles exactly four SRP capabilities: Lifecycle, Materializer, Wire-codec, Delivery.

## Consequences

- Multiple providers can populate one unified store; readers see no per-fabric difference.
- Per-fabric quirks (provenance, enforcement, derived-vs-enumerated) hide behind the materialization seam.
- Threads are a store entity the materializer derives — not a fabric concept.
- The single-writer daemon is the direct fix for multi-writer corruption.
- The store extends real state.db tables (project_meta, profiles, peer_sessions, inbox, agent_status); only 'threads' is genuinely new.

## Open Tail

- Thread keying across fabrics (root id vs synthesized hash vs subject).
- Write-reflection timing: optimistic vs echo (related to publish_signed_checked).
- The Codec trait still returns nostr_sdk types — a future transport-agnostic trait would need its own envelope types.

## Evidence

*(no verified line ranges)*

