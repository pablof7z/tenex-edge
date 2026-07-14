---
type: noun-entry
slug: session-identity
name: "session identity"
origin: extracted
source_refs:
  - transcript:2404-2407
  - transcript:2609-2626
---

# session identity

The authoritative pubkey a session signs and is routed by. Ordinary signers are
reconstructable from the machine management key plus a random non-secret salt
stored under that pubkey. A leased handle such as `quill-codex` is its public
alias; runtime and harness ids are local locators, not cryptographic inputs.
