---
type: noun-entry
slug: mint-session-identity
name: "mint_session_identity"
origin: extracted
source_refs:
  - transcript:1252-1275
  - transcript:2336-2345
---

# mint_session_identity

Mints (or deterministically re-derives) a session's own keypair via derive_session_keys_v2(management_secret, session_id). The management key is the per-machine root; a resumed session re-derives the identical pubkey. Records the minted pubkey into the append-only identities cache so later #p-tagged mentions resolve back to the right session.
