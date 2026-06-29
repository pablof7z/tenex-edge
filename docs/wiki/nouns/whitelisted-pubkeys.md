---
type: noun-entry
slug: whitelisted-pubkeys
name: "whitelisted_pubkeys"
origin: extracted
source_refs:
  - transcript:94-103
---

# whitelisted_pubkeys

A human operator's Nostr public keys, read from ~/.tenex-edge/config.json (JSON key `whitelistedPubkeys`). The source of truth for who is an admin in every project group via NIP-29 membership; distinct from the backend key, not derived from `user_nsec` or `tenex_private_key`.
