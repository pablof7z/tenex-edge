---
type: noun-entry
slug: per-session-pubkey-model
name: "per-session pubkey model"
origin: extracted
source_refs:
  - transcript:1068-1070
  - transcript:1122-1123
---

# per-session pubkey model

The pubkey is the authoritative ordinary-session identity. A fresh session
allocates a random, non-secret salt and derives its signer from that salt plus
the per-machine management secret. The persisted pubkey-to-salt binding lets the
backend reconstruct the signer for resume and route directly by pubkey.
