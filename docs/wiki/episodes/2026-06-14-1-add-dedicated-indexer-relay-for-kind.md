---
type: episode-card
date: 2026-06-14
session: ab43967d-95d5-49fd-aaf6-4bc65d80774e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/ab43967d-95d5-49fd-aaf6-4bc65d80774e.jsonl
salience: architecture
status: active
subjects:
  - indexer-relay
  - kind0-profile
  - relay-configuration
supersedes: []
related_claims: []
source_lines:
  - 1-1
  - 399-399
  - 528-539
  - 549-552
captured_at: 2026-06-18T00:20:34Z
---

# Episode: Add dedicated indexer relay for kind:0 profile publishing and lookup

## Prior State

tenex-edge connected only to the relays in cfg.relays (default wss://nip29.f7z.io); no dedicated relay for kind:0 profile events. Profile lookups and publishes went exclusively to that general relay pool.

## Trigger

User directive: tenex-edge needs to publish kind:0s to an indexer relay (default purplepag.es), configurable on config, and must also check that relay for kind:0 info (e.g. tenex-edge who).

## Decision

Added indexer_relay field to Config (default wss://purplepag.es, overridable via indexerRelay in config.json). At daemon startup, the transport connects to cfg.relays ∪ cfg.indexer_relay (deduped). provider_instance continues hashing only cfg.relays — canonical IDs unchanged. The indexer relay receives all publishes (kind:0 accepted; others silently rejected by the relay) and is queried on profile lookups.

## Consequences

- kind:0 profile events now reach purplepag.es, making agent identities discoverable via the standard Nostr profile indexer
- tenex-edge who lookups query purplepag.es for peer profile data
- Indexer relay is user-configurable without touching the operational relay pool
- provider_instance identity is preserved — adding an indexer relay does not rekey or re-hash the provider
- Non-profile events sent to purplepag.es are harmlessly rejected by that relay

## Open Tail

- Pre-existing test gaps in cli.rs (WhoRow attachable field) remain unfixed, blocking test compilation

## Evidence

- transcript lines 1-1
- transcript lines 399-399
- transcript lines 528-539
- transcript lines 549-552

