---
type: episode-card
date: 2026-06-09
session: f9bdcf4c-c972-46ff-91b8-9e30785d3331
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f9bdcf4c-c972-46ff-91b8-9e30785d3331.jsonl
salience: reversal
status: superseded
subjects:
  - default-relay
  - nip29
  - nostr-relay
supersedes: []
related_claims: []
source_lines:
  - 453-593
captured_at: 2026-06-17T23:50:24Z
---

# Episode: Migrate default relay from relay.tenex.chat to nip29.f7z.io

## Prior State

DEFAULT_RELAY constant was wss://relay.tenex.chat; live config had no explicit relays key (relying on the compiled default)

## Trigger

User directive: 'now that we're using nip29 we should be publishing to nip29.f7z.io'

## Decision

Changed DEFAULT_RELAY in src/config.rs to wss://nip29.f7z.io; added explicit relays array to ~/.tenex/config.json; updated test assertions to use the constant rather than hard-coded string

## Consequences

- All new installs/sessions default to nip29.f7z.io
- Live config is now explicit rather than relying on the compiled default
- Test assertions reference DEFAULT_RELAY constant instead of inline string, preventing future drift

## Open Tail

*(none)*

## Evidence

- transcript lines 453-593

