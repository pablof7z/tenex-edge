---
type: noun-entry
slug: agent-identity
name: "agent identity"
origin: extracted
source_refs:
  - transcript:236-240
---

# agent identity

The Nostr pubkey is authoritative. Ordinary sessions allocate their own pubkey
and public handle (`@<session-code>-<agent-slug>`). Agents configured with
`perSessionKey:false` deliberately reuse the configured pubkey across sequential
runs. Channel trust remains NIP-29 membership, independent of identity mode.
