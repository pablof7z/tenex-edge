---
type: noun-entry
slug: per-session-pubkey-nsec-derivation
name: "per-session pubkey (nsec derivation)"
origin: extracted
source_refs:
  - transcript:1068-1071
  - transcript:1122-1124
---

# per-session pubkey (nsec derivation)

For an ordinary session, `nsec` is derived from the per-machine management
secret and a random, non-secret signer salt. The salt is stored once in
`session_signers(pubkey, signer_salt)`, making the pubkey the reconstruction key
and keeping runtime locators out of the cryptographic identity.
