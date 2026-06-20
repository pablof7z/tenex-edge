---
type: episode-card
date: 2026-06-09
session: 98f9939c-f42b-43dd-baba-d9a176d4b2d7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/98f9939c-f42b-43dd-baba-d9a176d4b2d7.jsonl
salience: architecture
status: active
subjects:
  - relay-architecture
  - nip29-group-lifecycle
  - presence-delivery
supersedes:
  - 2026-06-09-2-migrate-default-relay-from-relay-tenex
related_claims: []
source_lines:
  - 2204-2213
  - 2215-2231
  - 2422-2434
  - 2436-2494
  - 2823-2846
  - 2893-2898
captured_at: 2026-06-17T23:52:07Z
---

# Episode: NIP-29 uses two distinct relays — presence on auth-gated relay, group management on NIP-29 relay

## Prior State

Assumed presence/activity/who events flowed through NIP-29 group semantics on a single relay; the `h` tag was assumed to carry NIP-29 group semantics everywhere

## Trigger

User asked 'is who and all that presence stuff using the nip29 stuff too?' then challenged the answer with 'are you sure?'; empirical relay probing showed zero kind:30315 presence events with h=tenex-edge on either relay

## Decision

Two distinct relay roles confirmed: `relay.tenex.chat` is a standard auth-gated (NIP-42) Nostr relay where `h` is namespace filtering only — no NIP-29 group enforcement. `nip29.f7z.io` is the true NIP-29 relay for group management (kind:9007, 9002, 39000). The same `h` tag serves different semantics on each relay. Group auto-creation via `ensure_group_and_membership` on session start does work — the earlier absence was caused by a stale installed daemon binary, not missing functionality.

## Consequences

- When daemon connects to nip29.f7z.io, presence events require the group to exist first — handled by ensure_group_and_membership before spawn_session
- Project list/edit target nip29.f7z.io specifically (kind:39000 fetch, kind:9002 publish)
- Migrating the daemon's default relay from relay.tenex.chat to nip29.f7z.io means all fabric events (presence, activity, mentions) now flow through the NIP-29 relay, requiring group membership for write access
- Daemon must complete group creation + membership before any presence events can be accepted by the relay

## Open Tail

- Whether to keep two relay roles (presence on one, groups on another) or unify on nip29.f7z.io after group creation is complete

## Evidence

- transcript lines 2204-2213
- transcript lines 2215-2231
- transcript lines 2422-2434
- transcript lines 2436-2494
- transcript lines 2823-2846
- transcript lines 2893-2898

