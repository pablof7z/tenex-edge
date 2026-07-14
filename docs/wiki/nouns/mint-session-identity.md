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

Selects a session's signing identity before managed spawn. Ordinary sessions atomically reserve a random non-secret signer salt, derived pubkey, public handle, and identity binding; resumed sessions reconstruct from the salt stored under that pubkey. Durable agents use their configured key and claim the one-active-run slot. Runtime row ids and harness-native locators are not signing inputs.
