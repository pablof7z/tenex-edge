---
type: noun-entry
slug: per-session-key
name: "per-session key"
origin: extracted
source_refs:
  - transcript:1068-1068
  - transcript:1122-1122
---

# per-session key

An ordinary session allocates a random, non-secret signer salt and derives its
key from that salt plus the per-machine management secret. The salt is stored
under the resulting authoritative pubkey, allowing signer reconstruction without
using a runtime row id or harness resume token.
