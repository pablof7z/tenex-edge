---
type: episode-card
date: 2026-06-09
session: ab9998c4-6e65-410e-b298-122a2072171c
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/ab9998c4-6e65-410e-b298-122a2072171c.jsonl
salience: architecture
status: superseded
subjects:
  - fabric-provider
  - codec-replacement
  - cqrs-store
supersedes:
  - 2026-06-09-1-read-model-is-the-contract-provider
related_claims: []
source_lines:
  - 1-164
  - 6116-6228
captured_at: 2026-06-18T00:02:10Z
---

# Episode: Replace Codec seam with Fabric Provider architecture

## Prior State

The Codec trait fused three unrelated concerns — wire mapping (domain event ↔ envelope), subscription model (filters → Vec<Filter>, relay-REQ-shaped), and access control (NIP-29 group ops bolted into kind1) — into a single trait. Readers had to know which kind/tag/group/relay produced data; there was no unified read-model.

## Trigger

docs/fabric-architecture.md proposal (9-phase behavior-preserving rewrite) + user directive to implement all phases 0–8 gated

## Decision

Adopted the Fabric Provider pattern: each provider (Kind1Nip29) bundles delivery, wire codec, materializer, and lifecycle into one struct that projects everything into canonical store rows. CQRS: all reads go through the store; nothing in a read path names a kind, tag, group, or relay. Legacy tables retained as 'deliberately retained storage'. Dual-write (canonical + legacy) during transition. NIP-10 thread grouping via native_thread_key in thread_origins. Single-writer invariant: provider holds Arc<Mutex<Store>> — never its own Connection.

## Consequences

- Codec trait's SubScope struct and filters method deleted; scope_filters() is now a free function on Scope
- Kind1Nip29Provider owns delivery (NostrDelivery), wire (Kind1WireCodec), materialization, and lifecycle (group_create/lock_closed/put_user moved from codec/kind1.rs to fabric/nip29/lifecycle.rs)
- handle_incoming became thin dispatch to provider.materialize(); fetch_mentions_into_inbox routes through provider.catch_up_mentions
- DaemonState dropped codec and delivery fields, replaced with single provider field
- rpc_send_message now uses provider.send(Intent) → OutboundReceipt; local delivery uses receipt.native_event_id
- 20 freeze tests pin existing behavior; all stayed green across all 8 phases
- Dual-write dedup verified: record_message idempotent on native_event_id, add_message_recipient idempotent on PK
- New CLI threads command with --project and --thread flags; new RPC endpoints rpc_list_threads, rpc_messages, rpc_thread_meta

## Open Tail

- Startup backfill on populated DB (migrating legacy rows) is smoke-tested but not yet battle-tested on large databases
- Legacy tables (inbox, project_meta) are 'deliberately retained' — eventual cleanup path undefined
- Thread conversation between agents e2e tested with real claude+codex sessions on isolated daemon

## Evidence

- transcript lines 1-164
- transcript lines 6116-6228

