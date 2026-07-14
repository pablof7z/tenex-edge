---
type: noun-entry
slug: per-session-pubkey-new-identity-model
name: "per-session pubkey (new identity model)"
origin: extracted
source_refs:
  - transcript:1068-1070
  - transcript:1077-1077
  - transcript:1122-1124
---

# per-session pubkey (new identity model)

An ordinary session derives its signer with HKDF from the per-machine management
secret and a random, non-secret salt. The salt is persisted under the resulting
authoritative pubkey. Runtime ids and harness resume tokens are locators only;
they do not participate in signer derivation.
