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

No base agent key exists. Every session mints its own keypair: nsec = derive(mgmt_secret, session_id). mgmt_secret is per-machine, so the same session_id on two machines yields different keys. Nothing is stored as a secret except the mgmt key; an append-only pubkey→session_id map lets the backend recognize, route, and resume.
