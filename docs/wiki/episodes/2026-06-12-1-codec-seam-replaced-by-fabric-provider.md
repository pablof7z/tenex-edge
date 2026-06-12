---
type: episode-card
date: 2026-06-12
session: ab9998c4-6e65-410e-b298-122a2072171c
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/ab9998c4-6e65-410e-b298-122a2072171c.jsonl
salience: architecture
status: superseded
subjects:
  - fabric-provider
  - codec-seam-removal
  - cqrs-read-model
  - single-writer-invariant
supersedes: []
related_claims: []
source_lines:
  - 3-19
  - 6116-6228
captured_at: 2026-06-12T19:38:53Z
---

# Episode: Codec seam replaced by Fabric Provider architecture

## Prior State

The Codec trait fused three unrelated concerns — wire mapping (domain event ↔ envelope), subscription model (filters → Vec<Filter>, relay-REQ-shaped), and access control (NIP-29 group create/lock/put-user bolted into kind1) — into a single seam. Read paths named kinds, tags, groups, and relays directly.

## Trigger

fabric-architecture.md proposal identified the Codec fusion as the core problem: 'The current Codec seam swaps NIP layouts, not fabrics. It traffics in nostr_sdk types and fuses three unrelated concerns into one trait.'

## Decision

Adopted the Fabric Provider pattern: all data is read from one unified local store; how it was hydrated is irrelevant to its use. A Fabric Provider (Kind1Nip29, future MLS/A2A) is a write-side materializer that owns wire shape, membership/ACL, and lifecycle side-effects, projecting everything into canonical store rows. Readers query the store; nothing in a read path ever names a kind, tag, group, or relay. CQRS with single-writer invariant (daemon owns the only rusqlite::Connection). Dual-write to legacy tables for safe transition.

## Consequences

- SubScope struct and Codec::filters() deleted; Scope struct replaces it for subscription construction
- Kind1Nip29Provider bundles delivery, wire codec, materializer, and NIP-29 lifecycle into one provider
- RawEnvelope(Nostr(Event)) is the transport-agnostic wire abstraction
- NIP-10 root e-tag grouping used for thread materialization (native_thread_key)
- Canonical tables (projects, threads, messages, message_recipients, membership, inbound_quarantine) now the read-model source-of-truth
- Dual-write writes canonical rows alongside legacy inbox for safe transition; idempotent on native_event_id
- 20 freeze tests pin existing behavior across all 9 phases (never regressed)
- Kind1Codec::filters logic relocated verbatim to scope_filters(); group_create/lock_closed/put_user promoted to fabric/nip29/lifecycle.rs

## Open Tail

- Legacy tables (inbox, project_meta) retained as 'deliberately retained' storage — final migration path not yet decided
- Additional providers (MLS, A2A) not yet implemented

## Evidence

- transcript lines 3-19
- transcript lines 6116-6228

