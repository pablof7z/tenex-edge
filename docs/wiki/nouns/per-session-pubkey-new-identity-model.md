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

Every session mints its own keypair; nsec = HKDF(mgmt_secret, session_id). mgmt_secret is per-machine, so the same session_id on two machines yields different keys. No base agent key, no ordinals, no occupancy/reservation logic. Only the mgmt key and an append-only pubkey→session_id map are stored.
