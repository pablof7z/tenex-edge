---
title: Tenex-Edge Transport Codec
slug: tenex-edge-transport-codec
topic: tenex-edge
summary: Envelope encoding and decoding is modularized as a codec set providing per-event encode, decode, and subscribe operations, decoupling envelope shapes from busin
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-12
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:d208c058-7b2b-4ff8-bb82-d63623d51097
  - session:36cc4546-228e-4d07-a1a8-9d0cd7cd5a6c
  - session:98f9939c-f42b-43dd-baba-d9a176d4b2d7
  - session:0bc06206-1f30-4e35-8373-f31d0f5c1dcc
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:40a4d401-2520-4781-b747-b0ef19594bed
  - session:cd74a605-9f83-4e21-a885-4d900e88ce07
---

# Tenex-Edge Transport Codec

## Codec Set Architecture

Envelope encoding and decoding is modularized as a codec set providing per-event encode, decode, and subscribe operations, decoupling envelope shapes from business logic so that alternative transports (such as another NIP-29 relay policy or Marmot/MLS) can be added as additional codec adapters without modifying domain logic. The codec set is granular per event type rather than a monolith. Heartbeat activity is included in the codec set because its shape varies across different codecs. The initial shape adapter uses kind:1 notes and kind:30315 heartbeats/statuses, all project-scoped with the NIP-29 `h` tag.

The `Codec` trait defines four verbs: `name` (stable wire-shape identifier), `encode` (domain to unsigned wire envelope), `decode` (wire to domain, or None if unrecognized), and `filters` (subscriptions fetching everything the codec speaks). A codec's `filters` verb must produce subscriptions covering every event type its `decode` can recognize. A new codec inherits a fixed domain taxonomy of nouns (Profile, Presence, Activity, Status, Mention, Proposal) and must supply a wire mapping for them. Any codec that reuses a kind across two domain events must define its own tie-breaker disambiguation rules. Domain code must never build wire events inline; all domain verbs (including rpc_user_prompt for Mention and rpc_propose for Proposal) must route through the codec set so that encoding is uniform and swappable.

Domain code must never name a specific Nostr kind, tag, or wire-protocol concept, keeping the domain layer transport-agnostic. The domain layer (`domain.rs` and `SubScope`) is genuinely transport-agnostic and names no kind, tag, or relay, exposing only the domain nouns (Profile, Presence, Activity, Status, Mention, Proposal).

The initial Nostr codec maps these nouns as follows: Profile → kind:0 with content `{"name": slug}` and tags `["host", host]` and `["p", owner]`; Presence → kind:30315 (NIP-38) with tags `["d", "tenex-edge-presence:<session>"], ["h", project], ["session-id", id], ["p", audience]`; Activity → kind:1 with tags `["h", project]` only (no agent tag); Status → kind:30315 with tags `["h", project], ["d", project]`; Mention → kind:1 with tags `["p", to_pubkey], ["h", project]`, plus optional `["session-id"/"from-session"]` — also with no agent tag; Proposal → kind:30023 with tags `["d", …], ["title", …], ["h", project], ["e", …]`. In the NIP-29 codec, an inbox reply publishes a kind:1 event that e-tags the original sender's mention event and p-tags the sender agent. The self-asserted `["agent", pk, slug]` tag has been removed from all event types (Presence, Status, Activity, Mention); it is no longer written or read, and no 'agent' tag must ever exist on kind:1 events. Slug is not carried on the wire; it is always resolved from the signer's kind:0 Profile event (`{"name": slug}`) by pubkey (empty on decode, resolved at routing time via `slug_for_pubkey`), for owners and agents identically with no special case. Activity and Mention both use kind:1 and are disambiguated on decode purely structurally by priority: if a `p` tag is present → Mention; else if an `e` tag with a root marker is present → TurnReply; otherwise → Activity — not by any identity or agent tag. Authorization for a directed Mention is by the signer's pubkey + NIP-29 group membership: admitted if signer ∈ hosted ∪ owners ∪ members(project); the relay's closed-group rules are defense-in-depth only. Profile, Presence, and Mention builders use `.allow_self_tagging()` to prevent nostr-sdk from stripping `p` tags equal to the author.

At the domain level, `fetch_mentions_into_inbox` skips events authored by operator keys (`state.owners`) rather than checking for an agent tag. The `suppress_inbox_event` guard is retained as a belt-and-suspenders check for any fetch that races the in-memory owners set.

Removing the `agent` tag is a breaking wire change: a non-upgraded daemon would classify mentions from this daemon as ambient Activity and not deliver them, so all peers must be upgraded together. The architecture document's codec taxonomy and wiki pages were updated to reflect the removal of the `agent` wire tag and the authorize-by-signer-pubkey model.

NIP-42 AUTH must be built into the transport layer from day one, since relays almost certainly require it for publishes and silently reject publishes without it. Additionally, transport must force NIP-42 AUTH completion (a warm-up fetch) before any subscribe, because relay.tenex.chat requires auth for reads and closes subscriptions opened before auth completes.

The `publish_signed` transport method publishes a B-signed event over an A-authed relay connection, and the event lands under B's authorship.

NIP-29 group management and group-state subscriptions should be properties of a nostr transport/ACL strategy rather than fused into the kind1 event codec.

Transport is built on the lean `nostr`/`nostr-sdk` stack behind a Transport trait, rather than embedding the full NMP kernel (which is unsuitable for headless CLI daemons), with NMP documented as the intended future swap-in behind the codec seam, not at the transport layer. However, the current `Codec` trait is coupled to nostr because its `encode`, `decode`, and `filters` signatures use `nostr_sdk` types (`EventBuilder`, `Event`, `Filter`), meaning it can only swap NIPs, not underlying transports. Furthermore, the `filters` verb bakes in relay REQ semantics, making it incompatible with push/gossip transports that use a different fetch model. To support any transport, the swap seam should be elevated to a `Fabric` trait that takes abstract `DomainEvent` and `SubScope` types, making `encode`/`decode`/`filters` private implementation details of a `NostrFabric`.

The runtime processes both deduped Event notifications and raw RelayPoolNotification::Message{Event} variants to avoid silent pending-discovery failures.

The `filters_cover_all_kinds_and_mentions` test is left untouched as the filter-equivalence oracle when `Kind1Codec::filters` delegates to the new `scope_filters`.

Codec integration is feature-complete for M1 with no immediate work needed.

The architecture report must include the NIP-29 codec event taxonomy: what tags are used, how addressing/tagging decisions are made, how the agent knows when the user reviewed the document, and an illustration with a potentially real flow.

<!-- citations: [^f3a73-36] [^f3a73-37] [^f3a73-38] [^f3a73-46] [^f3a73-58] [^f3a73-79] [^f3a73-85] [^f3a73-97] [^d208c-1] [^36cc4-6] [^98f99-30] [^0bc06-6] [^ab999-44] [^ab999-63] [^ab999-74] [^40a4d-6] [^40a4d-8] [^cd74a-7] -->
