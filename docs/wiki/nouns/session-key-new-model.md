---
type: noun-entry
slug: session-key-new-model
name: "session key (new model)"
origin: extracted
source_refs:
  - transcript:1068-1070
  - transcript:1122-1122
---

# session key (new model)

An ordinary session key is selected at start from the per-machine management
secret and a random, non-secret salt. Persisting the salt under the resulting
pubkey makes the signer reconstructable and resumable without treating any
runtime or harness id as identity.
