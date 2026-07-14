---
type: noun-entry
slug: session-as-identity-handle
name: "session (as identity handle)"
origin: extracted
source_refs:
  - transcript:547-547
  - transcript:1068-1068
  - transcript:1122-1122
---

# session (as identity handle)

A run id is private process-correlation state, not a signing identity, display
name, or chat target. The authoritative identity is the session pubkey; ordinary
signers reconstruct from pubkey-owned salt, while harness tokens and runtime ids
remain typed locators.
