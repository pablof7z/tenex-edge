---
type: episode-card
date: 2026-06-09
session: f9bdcf4c-c972-46ff-91b8-9e30785d3331
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f9bdcf4c-c972-46ff-91b8-9e30785d3331.jsonl
salience: product
status: active
subjects:
  - relay-config
  - nip29
supersedes:
  - 2026-06-12-2-nip29-f7z-io-added-to-app
related_claims: []
source_lines:
  - 453-455
  - 487-488
  - 539-594
captured_at: 2026-06-12T20:04:49Z
---

# Episode: Switch default relay from relay.tenex.chat to nip29.f7z.io

## Prior State

The compiled-in DEFAULT_RELAY was `wss://relay.tenex.chat`. The live ~/.tenex/config.json had no explicit `relays` field, relying entirely on the compiled default.

## Trigger

User directive: 'now that we're using nip29 we should be publishing to nip29.f7z.io'

## Decision

DEFAULT_RELAY in src/config.rs changed to `wss://nip29.f7z.io`. Live config updated with explicit `"relays": ["wss://nip29.f7z.io"]` so it no longer depends on the compiled-in default. Test assertions updated to reference the constant rather than hard-coded strings.

## Consequences

- All new installations will default to the nip29 relay
- Existing live deployment now explicitly targets nip29.f7z.io regardless of compiled default
- Test assertions now reference the DEFAULT_RELAY constant, preventing future relay changes from breaking tests silently

## Open Tail

*(none)*

## Evidence

- transcript lines 453-455
- transcript lines 487-488
- transcript lines 539-594

