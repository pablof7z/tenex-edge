---
type: episode-card
date: 2026-06-09
session: ab9998c4-6e65-410e-b298-122a2072171c
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/ab9998c4-6e65-410e-b298-122a2072171c.jsonl
salience: product
status: active
subjects:
  - relay-config
  - app-connectivity
  - default-relays
supersedes: []
related_claims: []
source_lines:
  - 5319-5349
captured_at: 2026-06-18T00:02:10Z
---

# Episode: Add fabric relay to app default relay set

## Prior State

App's default_relays() contained only public Nostr relays: relay.damus.io, nos.lol, nostr.wine, relay.primal.net — no nip29.f7z.io. The production daemon (where Claude agents run) publishes proposals and comments to f7z. App and daemon were on disjoint relay sets.

## Trigger

E2e test revealed the app could not see any proposals or comments from the daemon — the deep-linked proposal document opened blank because the app never subscribed to f7z where the daemon publishes

## Decision

Added wss://nip29.f7z.io to default_relays() in relay_config.rs, with the existing D3/D4 doctrine comment preserved (app-authored connectivity defaults, separate from user's NIP-65 relay list, Rust is single writer)

## Consequences

- App can now fetch and render proposals/comments published by daemon agents on f7z
- Deep-linked nostr: URIs resolve in the app when the content lives on f7z
- Committed as 79d00fe: 'Add nip29.f7z.io to default relays (tenex fabric reachability)'

## Open Tail

- Relay set is compile-time only — user cannot add/remove relays from the UI (D4: Rust is single writer)
- No relay health monitoring or fallback if f7z is down

## Evidence

- transcript lines 5319-5349

