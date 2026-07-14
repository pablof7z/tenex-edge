---
type: noun-entry
slug: identities-table
name: "identities (table)"
origin: extracted
source_refs:
  - transcript:696-701
---

# identities (table)

Maps an authoritative pubkey to local lifecycle, public-handle, and resume
locators. It contains no signer secret or derivation input; ordinary-session
reconstruction material is stored once in `session_signers(pubkey, signer_salt)`.
